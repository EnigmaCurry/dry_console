use std::process::Stdio;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::api::websocket::{handle_websocket, WebSocketResponse};
use crate::broadcast;
use crate::{api::route, AppRouter};
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use dry_console_dto::websocket::{
    ClientMsg, CloseCode, Process, ProcessComplete, ProcessOutput, ServerMsg, StreamType,
};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tracing::debug;
use ulid::Ulid;

pub fn main(shutdown: broadcast::Sender<()>) -> AppRouter {
    Router::new().merge(command_execute(shutdown))
}

#[utoipa::path(
    get,
    path = "/api/workstation/command_execute/",
    responses(
        (status = OK, description = "Open websocket connection to read executed command stdout")
    )
)]
fn command_execute(shutdown: broadcast::Sender<()>) -> AppRouter {
    #[derive(PartialEq, Clone, Debug)]
    enum State {
        AwaitingCommand,
        RunningProcess,
        Completed,
    }

    /// WebSocket connection handler
    async fn websocket(socket: WebSocket<ServerMsg, ClientMsg>, shutdown: broadcast::Receiver<()>) {
        let state = Arc::new(Mutex::new(State::AwaitingCommand));
        let socket = Arc::new(Mutex::new(Some(socket)));

        // Move ping handling into the handle_websocket helper
        handle_websocket(socket.clone(), shutdown, move |msg| {
            let state = state.clone();
            let socket = socket.clone();
            Box::pin(async move {
                let mut state_ref = state.lock().await;
                let mut socket_ref = socket.lock().await;
                let socket_guard = socket_ref.as_mut().unwrap();
                match *state_ref {
                    State::AwaitingCommand => match msg {
                        Message::Item(ClientMsg::Command(_command_id)) => {
                            *state_ref = State::RunningProcess;
                            drop(state_ref); // Drop the lock on state to run the command

                            // Run the command asynchronously
                            let process_id = Ulid::new();
                            let script = r#"
                            #!/bin/sh
                            echo "Hii" >/dev/stderr
                            for i in $(seq 25); do
                               echo $i
                               echo "uhh" >/dev/stderr
                               sleep 0.1
                            done
                            "#;
                            let mut process = Command::new("/bin/sh")
                                .arg("-c")
                                .arg(script)
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                                .expect("Failed to start process");

                            // Send the client info about the process just started:
                            socket_guard
                                .send(Message::Item(ServerMsg::Process(Process {
                                    id: process_id,
                                })))
                                .await
                                .ok();
                            // Handle stdout and stderr concurrently
                            let stdout = process.stdout.take().expect("Failed to take stdout");
                            let stderr = process.stderr.take().expect("Failed to take stderr");

                            let stdout_reader = BufReader::new(stdout).lines();
                            let stderr_reader = BufReader::new(stderr).lines();

                            let mut stdout_stream =
                                tokio_stream::wrappers::LinesStream::new(stdout_reader);
                            let mut stderr_stream =
                                tokio_stream::wrappers::LinesStream::new(stderr_reader);

                            let mut stdout_stream = stdout_stream.fuse();
                            let mut stderr_stream = stderr_stream.fuse();

                            debug!("here");
                            let mut stdout_ended = false;
                            let mut stderr_ended = false;

                            loop {
                                tokio::select! {
                                    stdout_line = stdout_stream.next(), if !stdout_ended => {
                                        match stdout_line {
                                            Some(Ok(line_content)) => {
                                                debug!("stdout_line: {:?}", line_content);
                                                socket_guard
                                                    .send(Message::Item(ServerMsg::ProcessOutput(
                                                        ProcessOutput {
                                                            stream: StreamType::Stdout,
                                                            id: process_id,
                                                            line: line_content,
                                                        },
                                                    )))
                                                    .await
                                                    .ok();
                                            }
                                            Some(Err(e)) => {
                                                debug!("Error reading stdout: {:?}", e);
                                            }
                                            None => {
                                                debug!("stdout_stream ended, setting stdout_ended = true");
                                                stdout_ended = true;
                                            }
                                        }
                                    }
                                    stderr_line = stderr_stream.next(), if !stderr_ended => {
                                        match stderr_line {
                                            Some(Ok(line_content)) => {
                                                debug!("stderr_line: {:?}", line_content);
                                                socket_guard
                                                    .send(Message::Item(ServerMsg::ProcessOutput(
                                                        ProcessOutput {
                                                            stream: StreamType::Stderr,
                                                            id: process_id,
                                                            line: line_content,
                                                        },
                                                    )))
                                                    .await
                                                    .ok();
                                            }
                                            Some(Err(e)) => {
                                                debug!("Error reading stderr: {:?}", e);
                                            }
                                            None => {
                                                debug!("stderr_stream ended, setting stderr_ended = true");
                                                stderr_ended = true;
                                            }
                                        }
                                    }
                                    else => {
                                        debug!("Exiting the loop. stdout_ended: {}, stderr_ended: {}", stdout_ended, stderr_ended);
                                        if stdout_ended && stderr_ended {
                                            break;
                                        }
                                    }
                                }
                            }
                            debug!("done");                               // Wait for the process to finish
                            let status = process
                                .wait()
                                .await
                                .expect("Failed to wait on child process");
                            socket_guard
                                .send(Message::Item(ServerMsg::ProcessComplete(ProcessComplete {
                                    id: process_id,
                                    code: status.code().unwrap_or(128),
                                })))
                                .await
                                .ok();

                            *state.lock().await = State::Completed; // Update the state
                            Some(WebSocketResponse {
                                close: true,
                                close_code: CloseCode::GoingAway,
                                close_message: "Process finished".to_string(),
                            })
                        }
                        r => Some(WebSocketResponse {
                            close: true,
                            close_code: CloseCode::UnsupportedData,
                            close_message: format!("Received unexpected message: {r:?}"),
                        }),
                    },
                    State::RunningProcess | State::Completed => Some(WebSocketResponse {
                        close: true,
                        close_code: CloseCode::UnsupportedData,
                        close_message: "Received unexpected message.".to_string(),
                    }),
                }
            })
        })
        .await;
    }

    /// Upgrade HTTP connection to WebSocket
    async fn upgrade(
        ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
        shutdown: broadcast::Sender<()>,
    ) -> impl IntoResponse {
        let shutdown_rx = shutdown.subscribe();
        debug!("WebSocket upgrade request received.");
        ws.on_upgrade(move |socket| websocket(socket, shutdown_rx))
    }

    route(
        "/command_execute/",
        get(move |ws: WebSocketUpgrade<_, _>| upgrade(ws, shutdown.clone())),
    )
}
