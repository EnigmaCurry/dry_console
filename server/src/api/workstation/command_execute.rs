use std::process::Stdio;
use std::sync::Arc;

use crate::api::websocket::{handle_websocket, WebSocketResponse};
use crate::broadcast;
use crate::{api::route, AppRouter};
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use dry_console_dto::websocket::{ClientMsg, CloseCode, ProcessComplete, ProcessOutput, ServerMsg};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::Instant;
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
    #[derive(PartialEq, Clone)]
    enum State {
        AwaitingPong,
        AwaitingCommand,
        RunningProcess,
        Completed,
    }

    /// WebSocket connection handler
    async fn websocket(socket: WebSocket<ServerMsg, ClientMsg>, shutdown: broadcast::Receiver<()>) {
        let state = Arc::new(Mutex::new(State::AwaitingPong));
        let socket = Arc::new(Mutex::new(Some(socket)));

        {
            // Initial Ping
            let mut socket_guard = socket.lock().await;
            socket_guard
                .as_mut()
                .unwrap()
                .send(Message::Item(ServerMsg::Ping))
                .await
                .ok();
        }

        handle_websocket(socket.clone(), shutdown, move |msg| {
            let state = state.clone();
            let socket = socket.clone();
            Box::pin(async move {
                let mut state_ref = state.lock().await;
                let mut socket_ref = socket.lock().await;
                let socket_guard = socket_ref.as_mut().unwrap();

                match *state_ref {
                    State::AwaitingPong => match msg {
                        Message::Item(ClientMsg::Pong) => {
                            *state_ref = State::AwaitingCommand;
                            Some(WebSocketResponse {
                                close: false,
                                close_code: CloseCode::NormalClosure,
                                close_message: "Awaiting command".to_string(),
                            })
                        }
                        _ => Some(WebSocketResponse {
                            close: true,
                            close_code: CloseCode::UnsupportedData,
                            close_message: "Received unexpected message.".to_string(),
                        }),
                    },
                    State::AwaitingCommand => match msg {
                        Message::Item(ClientMsg::Command(_command_id)) => {
                            *state_ref = State::RunningProcess;
                            drop(state_ref); // Drop the lock on state to run the command

                            // Run the command asynchronously
                            let process_id = Ulid::new();
                            let _start = Instant::now();
                            let mut process = Command::new("seq")
                                .arg("10")
                                .stdout(Stdio::piped())
                                .spawn()
                                .expect("Failed to start process");

                            // Handle the stdout
                            if let Some(stdout) = process.stdout.take() {
                                let reader = BufReader::new(stdout);
                                let mut reader_stream =
                                    tokio_stream::wrappers::LinesStream::new(reader.lines());

                                while let Some(line_result) = reader_stream.next().await {
                                    match line_result {
                                        Ok(line_content) => {
                                            socket_guard
                                                .send(Message::Item(ServerMsg::ProcessOutput(
                                                    ProcessOutput {
                                                        id: process_id,
                                                        line: line_content,
                                                    },
                                                )))
                                                .await
                                                .ok();
                                        }
                                        Err(e) => {
                                            eprintln!("Error reading line: {}", e);
                                        }
                                    }
                                }
                            }

                            // Wait for the process to finish
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
                                close: false,
                                close_code: CloseCode::NormalClosure,
                                close_message: "Running process".to_string(),
                            })
                        }
                        _ => Some(WebSocketResponse {
                            close: true,
                            close_code: CloseCode::UnsupportedData,
                            close_message: "Received unexpected message.".to_string(),
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
