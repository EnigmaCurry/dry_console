use crate::api::websocket::{handle_websocket, WebSocketResponse};
use crate::api::workstation::command::CommandLibrary;
use crate::app_state::SharedState;
use crate::broadcast;
use crate::{api::route, AppRouter};
use axum::extract::State;
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use dry_console_dto::websocket::{
    ClientMsg, CloseCode, Process, ProcessComplete, ProcessOutput, ServerMsg, StreamType,
};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};
use ulid::Ulid;
const TIMEOUT_INTERVAL: u64 = 2000;

pub fn main(shutdown: broadcast::Sender<()>, state: State<SharedState>) -> AppRouter {
    Router::new().merge(command_execute(shutdown, state))
}

#[utoipa::path(
    get,
    path = "/api/workstation/command_execute/",
    responses(
        (status = OK, description = "Open websocket connection to read executed command stdout")
    )
)]
fn command_execute(shutdown: broadcast::Sender<()>, state: State<SharedState>) -> AppRouter {
    #[derive(PartialEq, Clone, Debug)]
    enum SocketState {
        AwaitingCommand,
        RunningProcess,
        Completed,
    }

    /// WebSocket connection handler
    async fn websocket(
        socket: WebSocket<ServerMsg, ClientMsg>,
        shutdown: broadcast::Receiver<()>,
        State(shared_state): State<SharedState>,
    ) {
        let state = Arc::new(Mutex::new(SocketState::AwaitingCommand));
        let socket = Arc::new(Mutex::new(Some(socket))); // Ensure `socket` is Arc<Mutex<...>>

        let (cancel_tx, cancel_rx) = watch::channel(false);

        handle_websocket(socket.clone(), shutdown, move |msg| {
            info!("WebSocket open!");
            let state = state.clone();
            let socket = socket.clone(); // Clone the Arc for use in the spawned task
            let mut cancel_rx = cancel_rx.clone();
            let cancel_tx = cancel_tx.clone();
            let mut timeout_interval = tokio::time::interval(Duration::from_millis(TIMEOUT_INTERVAL));
            let shared_state = shared_state.clone();
            Box::pin(async move {
                let mut state_ref = state.lock().await;
                match *state_ref {
                    SocketState::AwaitingCommand => match msg {
                        Message::Item(ClientMsg::Command(command)) => {
                            *state_ref = SocketState::RunningProcess;
                            drop(state_ref); // Drop the lock on state to run the command
                            let process_id = Ulid::new();
                            let command_library = &shared_state.read().await.command_library.clone();
                            let command = match CommandLibrary::from_id(command.id, command_library.clone()).await {
                                Some(c) => c,
                                None => {
                                    error!("Failed to get script entry: {}", command.id);
                                    return None;
                                },
                            };
                            let script;
                            {
                                let shared_state = shared_state.read().await;
                                //debug!("command_script: {:?}", shared_state.command_script.clone());
                                script = command.get_script(&shared_state.command_id, &shared_state.command_script);
                            }
                            let mut process = Command::new("/bin/bash")
                                .arg("-c")
                                .arg(script)
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()
                                .expect("Failed to start process");

                            {
                                let mut socket_ref = socket.lock().await;
                                let socket_guard = socket_ref.as_mut().unwrap();
                                socket_guard
                                    .send(Message::Item(ServerMsg::Process(Process {
                                        id: process_id,
                                    })))
                                    .await
                                    .ok();
                            }

                            let stdout = process.stdout.take().expect("Failed to take stdout");
                            let stderr = process.stderr.take().expect("Failed to take stderr");

                            let stdout_reader = BufReader::new(stdout).lines();
                            let stderr_reader = BufReader::new(stderr).lines();
                            tokio::spawn({
                                let socket = socket.clone(); // Clone the Arc again for the task
                                async move {
                                    let mut stdout_stream = tokio_stream::wrappers::LinesStream::new(stdout_reader).fuse();
                                    let mut stderr_stream = tokio_stream::wrappers::LinesStream::new(stderr_reader).fuse();

                                    let mut stdout_ended = false;
                                    let mut stderr_ended = false;

                                    loop {
                                        tokio::select! {
                                            _ = timeout_interval.tick() => {
                                                if stdout_ended && stderr_ended {
                                                    break;
                                                }                                             },
                                            stdout_line = stdout_stream.next(), if !stdout_ended => {
                                                match stdout_line {
                                                    Some(Ok(line_content)) => {
                                                        let mut socket_ref = socket.lock().await;
                                                        let socket_guard = socket_ref.as_mut().unwrap();
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
                                                        eprintln!("Error reading stdout: {:?}", e);
                                                    }
                                                    None => {
                                                        stdout_ended = true;
                                                    }
                                                }
                                            }
                                            stderr_line = stderr_stream.next(), if !stderr_ended => {
                                                match stderr_line {
                                                    Some(Ok(line_content)) => {
                                                        let mut socket_ref = socket.lock().await;
                                                        let socket_guard = socket_ref.as_mut().unwrap();
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
                                                        eprintln!("Error reading stderr: {:?}", e);
                                                    }
                                                    None => {
                                                        stderr_ended = true;
                                                    }
                                                }
                                            }
                                            _ = cancel_rx.changed() => {
                                                if *cancel_rx.borrow() {
                                                    process.kill().await.expect("Failed to kill process");
                                                    let mut socket_ref = socket.lock().await;
                                                    if let Some(mut socket_guard) = socket_ref.take() {
                                                        socket_guard
                                                            .send(Message::Item(ServerMsg::ProcessComplete(ProcessComplete {
                                                                id: process_id,
                                                                code: 128,
                                                            })))
                                                            .await
                                                            .ok();
                                                        if let Some(socket_guard) = socket_ref.take() {
                                                            let _ = socket_guard.close().await;
                                                            debug!("WebSocket closed!");
                                                        } else {
                                                            debug!("WebSocket already closed!");
                                                        }
                                                    }
                                                    break;
                                                }
                                            }
                                            else => {
                                                if stdout_ended && stderr_ended {
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    let status = process.wait().await.expect("Failed to wait on child process");
                                    let mut socket_ref = socket.lock().await;
                                    if let Some(mut socket_guard) = socket_ref.take() {
                                        socket_guard
                                            .send(Message::Item(ServerMsg::ProcessComplete(ProcessComplete {
                                                id: process_id,
                                                code: status.code().unwrap_or(128),
                                            })))
                                            .await
                                            .ok();
                                        if let Some(socket_guard) = socket_ref.take() {
                                            let _ = socket_guard.close().await;
                                            debug!("WebSocket closed!");
                                        } else {
                                            debug!("WebSocket already closed!");
                                        }
                                    }

                                    let mut state_ref = state.lock().await;
                                    *state_ref = SocketState::Completed;
                                }
                            });

                            None
                        }
                        Message::Item(ClientMsg::Cancel) => {
                            let _ = cancel_tx.send(true);
                            *state.lock().await = SocketState::Completed;
                            None
                        }
                        r => Some(WebSocketResponse {
                            close: true,
                            close_code: CloseCode::UnsupportedData,
                            close_message: format!("Received unexpected message: {r:?}"),
                        }),
                    },
                    SocketState::RunningProcess => {
                        match msg {
                            Message::Item(ClientMsg::Cancel) => {
                                let _ = cancel_tx.send(true);
                                *state.lock().await = SocketState::Completed;
                                None
                            },
                            m => {
                                Some(WebSocketResponse {
                                    close: true,
                                    close_code: CloseCode::UnsupportedData,
                                    close_message: format!("Received unexpected message: {:?}", m),
                                })}
                        }
                    },
                    SocketState::Completed => Some(WebSocketResponse {
                        close: true,
                        close_code: CloseCode::UnsupportedData,
                        close_message: format!("Received unexpected message: {:?}", msg),
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
        state: State<SharedState>,
    ) -> impl IntoResponse {
        let shutdown_rx = shutdown.subscribe();
        debug!("WebSocket upgrade request received.");
        ws.on_upgrade(move |socket| websocket(socket, shutdown_rx, state))
    }

    route(
        "/command_execute/",
        get(move |ws: WebSocketUpgrade<_, _>| upgrade(ws, shutdown.clone(), state)),
    )
}
