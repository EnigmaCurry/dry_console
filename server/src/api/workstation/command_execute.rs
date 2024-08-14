use crate::api::websocket::{handle_websocket, WebSocketResponse};
use crate::broadcast;
use crate::{api::route, AppRouter};
use axum::{response::IntoResponse, routing::get, Router};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use dry_console_dto::websocket::{ClientMsg, CloseCode, ServerMsg};
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
        #[derive(PartialEq)]
        enum State {
            AwaitingPong,
            AwaitingCommand,
            RunningProcess,
            Completed,
        }

        let start = Instant::now();
        let mut state = State::AwaitingPong;

        socket.send(Message::Item(ServerMsg::Ping)).await.ok();

        handle_websocket(socket, shutdown, move |msg| match state {
            State::AwaitingPong => match msg {
                Message::Item(ClientMsg::Pong) => {
                    state = State::AwaitingCommand;
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
                    state = State::RunningProcess;
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
