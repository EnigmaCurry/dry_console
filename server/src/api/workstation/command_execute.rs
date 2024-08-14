use std::io::BufRead;
use std::{io::BufReader, process::Command, process::Stdio};

use crate::broadcast;
use crate::{api::route, AppRouter};
use axum::extract::ws::CloseFrame;
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use dry_console_dto::websocket::{
    ClientMsg, CloseCode, PingReport, Process, ProcessComplete, ProcessOutput, ServerMsg,
};
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

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
    #[derive(PartialEq)]
    enum State {
        AwaitingPong,
        AwaitingCommand,
        RunningProcess,
        Completed,
    }
    /// state machine websocket is spawned once per connection:
    async fn websocket(
        mut socket: WebSocket<ServerMsg, ClientMsg>,
        mut shutdown: broadcast::Receiver<()>,
    ) {
        // Record time of start connection:
        let start = Instant::now();
        let mut close_code = CloseCode::NormalClosure;
        let mut close_message = "Goodbye.".to_string();
        // Send initial ping:
        socket.send(Message::Item(ServerMsg::Ping)).await.ok();
        let mut state = State::AwaitingPong;
        // Receive messages indefinitely:
        loop {
            tokio::select! {
                Some(msg) = socket.recv() => {
                    match msg {
                        Ok(item) => match state {
                            State::AwaitingPong => match item {
                                Message::Item(ClientMsg::Pong) => {
                                    state = State::AwaitingCommand;
                                    socket
                                        .send(Message::Item(ServerMsg::PingReport(PingReport { duration_ms: start.elapsed() })))
                                        .await
                                        .ok();
                                }
                                _ => {
                                    close_code = CloseCode::UnsupportedData;
                                    close_message = "Received unexpected message.".to_string();
                                    warn!(close_message);
                                    break;
                                }
                            },
                            State::AwaitingCommand => match item {
                                Message::Item(ClientMsg::Command(_command_id)) => {
                                    let process_id = ulid::Ulid::new();
                                    state = State::RunningProcess;
                                    socket
                                        .send(Message::Item(ServerMsg::Process(Process { id: process_id })))
                                        .await
                                        .ok();
                                    // Run the command:
                                    let mut process = Command::new("seq")
                                        .arg("10")
                                        .stdout(Stdio::piped())
                                        .spawn()
                                        .expect("Failed to start process");
                                    // Handle the stdout
                                    if let Some(stdout) = process.stdout.take() {
                                        let reader = BufReader::new(stdout);
                                        for line in reader.lines() {
                                            match line {
                                                Ok(line_content) => {
                                                    // Process the line
                                                    socket
                                                        .send(Message::Item(ServerMsg::ProcessOutput(ProcessOutput { id: process_id, line: line_content })))
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
                                    let status = process.wait().expect("Failed to wait on child process");
                                    socket
                                        .send(Message::Item(ServerMsg::ProcessComplete(ProcessComplete {id: process_id, code: status.code().unwrap_or(128)})))
                                        .await
                                        .ok();
                                    break;
                                }
                                _ => {
                                    close_code = CloseCode::UnsupportedData;
                                    close_message = "Received unexpected message.".to_string();
                                    warn!(close_message);
                                    break;
                                }
                            },
                            State::RunningProcess | State::Completed => {
                                close_code = CloseCode::UnsupportedData;
                                close_message = "Received unexpected message.".to_string();
                                warn!(close_message);
                                break;
                            }
                        },
                        Err(err) => {
                            close_code = CloseCode::InvalidFramePayloadData;
                            close_message = "Error parsing message.".to_string();
                            error!("Got error: {}", err);
                            break;
                        }
                    }
                }
                _ = shutdown.recv() => {
                    close_code = CloseCode::GoingAway;
                    close_message = "Server is shutting down.".to_string();
                    break;
                }
            }
        }

        // Disconnect
        // Send the close frame to the client
        let close_frame = CloseFrame {
            code: close_code.into(),
            reason: close_message.into(),
        };
        if let Err(err) = socket.send(Message::Close(Some(close_frame))).await {
            eprintln!("Error sending close frame: {:?}", err);
        }

        // Now close the socket
        if let Err(err) = socket.close().await {
            eprintln!("Error closing the socket: {:?}", err);
        }
        info!("Closed websocket.");
    }
    async fn upgrade(
        ws: WebSocketUpgrade<ServerMsg, ClientMsg>,
        shutdown: broadcast::Sender<()>,
    ) -> impl IntoResponse {
        let shutdown_rx = shutdown.subscribe();
        debug!("Websocket upgrade request received.");
        ws.on_upgrade(move |socket| websocket(socket, shutdown_rx))
    }
    route(
        "/command_execute/",
        get(move |ws: WebSocketUpgrade<_, _>| upgrade(ws, shutdown.clone())),
    )
}
