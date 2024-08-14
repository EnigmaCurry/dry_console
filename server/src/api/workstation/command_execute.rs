use std::process::Stdio;
use std::sync::{Arc, Mutex};

use crate::api::websocket::{handle_websocket, WebSocketResponse};
use crate::broadcast;
use crate::{api::route, AppRouter};
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use dry_console_dto::websocket::{ClientMsg, CloseCode, ProcessComplete, ProcessOutput, ServerMsg};
use tokio::io::BufReader;
use tokio::process::Command;
use tokio::time::Instant;
use tracing::debug;

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
    /// state machine websocket is spawned once per connection:
    async fn websocket(
        mut socket: WebSocket<ServerMsg, ClientMsg>,
        mut shutdown: broadcast::Receiver<()>,
    ) {
        let start = Instant::now();
        let state = Arc::new(Mutex::new(State::AwaitingPong));

        socket.send(Message::Item(ServerMsg::Ping)).await.ok();

        handle_websocket(socket, shutdown, move |msg| {
            let state = state.clone();
            Box::pin({
                async move {
                    let mut state_ref = state.lock().unwrap();
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
                                let process_id = ulid::Ulid::new();
                                *state_ref = State::RunningProcess;
                                // Run the command asynchronously
                                // Simulate long running process here if needed
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
                }
            })
        })
        .await;
    }

    /// Upgrade HTTP connection to websocket:
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
