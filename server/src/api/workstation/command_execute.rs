use std::io::BufRead;
use std::{io::BufReader, process::Command, process::Stdio};

use crate::{
    api::{
        route,
        websocket::{ClientMsg, PingReport, Process, ServerMsg},
    },
    AppRouter,
};
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

pub fn main() -> AppRouter {
    Router::new().merge(command_execute())
}

#[utoipa::path(
    get,
    path = "/api/workstation/command_execute/",
    responses(
        (status = OK, description = "Open websocket connection to read executed command stdout")
    )
)]
fn command_execute() -> AppRouter {
    #[derive(PartialEq)]
    enum State {
        AwaitingPong,
        AwaitingCommand,
        RunningProcess,
        Completed,
    }
    /// state machine websocket is spawned once per connection:
    async fn websocket(mut socket: WebSocket<ServerMsg, ClientMsg>) {
        // record time of start connection:
        let start = Instant::now();
        let mut close_message = "Goodbye.".to_string();
        // Send initial ping:
        socket.send(Message::Item(ServerMsg::Ping)).await.ok();
        let mut state = State::AwaitingPong;
        // Receive messages indefinitely:
        while let Some(msg) = socket.recv().await {
            match msg {
                Ok(item) => match state {
                    State::AwaitingPong => match item {
                        Message::Item(ClientMsg::Pong) => {
                            state = State::AwaitingCommand;
                            socket
                                .send(Message::Item(ServerMsg::PingReport(PingReport {
                                    duration_ms: start.elapsed(),
                                })))
                                .await
                                .ok();
                        }
                        _ => {
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
                                .send(Message::Item(ServerMsg::Process(Process {
                                    id: process_id,
                                })))
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
                                            println!("{}", line_content);
                                        }
                                        Err(e) => {
                                            eprintln!("Error reading line: {}", e);
                                        }
                                    }
                                }
                            }

                            // Wait for the process to finish
                            let status = process.wait().expect("Failed to wait on child process");

                            if status.success() {
                                println!("Process completed successfully.");
                            } else {
                                eprintln!("Process failed with status: {:?}", status);
                            }
                        }
                        _ => {
                            close_message = "Received unexpected message.".to_string();
                            warn!(close_message);
                            break;
                        }
                    },
                    State::RunningProcess | State::Completed => {
                        close_message = "Received unexpected message.".to_string();
                        warn!(close_message);
                        break;
                    },
                },
                Err(err) => {
                    close_message = "Error parsing message.".to_string();
                    error!("Got error: {}", err);
                    break;
                }
            }
        }
        // Send Goodbye
        socket
            .send(Message::Item(ServerMsg::Goodbye(close_message.to_string())))
            .await
            .ok();
        // Disconnect
        info!("Closed websocket.");
    }

    async fn upgrade(ws: WebSocketUpgrade<ServerMsg, ClientMsg>) -> impl IntoResponse {
        debug!("Websocket upgrade request received.");
        ws.on_upgrade(websocket)
    }
    route("/command_execute/", get(upgrade))
}
