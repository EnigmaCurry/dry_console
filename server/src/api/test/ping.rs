use std::convert::Infallible;

use super::test_route;

use crate::{app_state::SharedState, AppRouter};
use axum::{
    response::{Html, IntoResponse},
    routing::{get, MethodRouter},
    Router,
};
use axum_typed_websockets::{Message, WebSocket, WebSocketUpgrade};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration, Instant};

pub fn main() -> AppRouter {
    Router::new().merge(ping()).merge(ws())
}

fn route(path: &str, method_router: MethodRouter<SharedState, Infallible>) -> AppRouter {
    test_route(super::TestModule::Ping, path, method_router)
}

#[utoipa::path(
    get,
    path = "/api/test/ping/ws/",
    responses(
        (status = OK, description = "Get websocket connection", body = str)
    )
)]
fn ws() -> AppRouter {
    // Send a ping and measure how long time it takes to get a pong back

    async fn ping_pong_socket(mut socket: WebSocket<ServerMsg, ClientMsg>) {
        println!("hi");
        let start = Instant::now();
        // Send a Ping message to the client
        if socket.send(Message::Item(ServerMsg::Ping)).await.is_ok() {
            // Wait for a response with a timeout
            match timeout(Duration::from_secs(5), socket.recv()).await {
                Ok(Some(Ok(Message::Item(ClientMsg::Pong)))) => {
                    println!("ping: {:?}", start.elapsed());
                }
                Ok(Some(Ok(_))) => {
                    println!("Received unexpected message");
                }
                Ok(Some(Err(err))) => {
                    eprintln!("Received error: {}", err);
                }
                Ok(None) => {
                    println!("Connection closed by client");
                }
                Err(_) => {
                    println!("Timeout waiting for Pong response");
                }
            }
        } else {
            eprintln!("Failed to send Ping message");
        }
    }
    async fn upgrade(ws: WebSocketUpgrade<ServerMsg, ClientMsg>) -> impl IntoResponse {
        ws.on_upgrade(ping_pong_socket)
    }
    route("/ws", get(upgrade))
}

#[derive(Debug, Serialize)]
enum ServerMsg {
    Ping,
}

#[derive(Debug, Deserialize)]
enum ClientMsg {
    Pong,
}

#[utoipa::path(
    get,
    path = "/api/test/ping/",
    responses(
        (status = OK, description = "Test client for websocket", body = str)
    )
)]
fn ping() -> AppRouter {
    async fn handler() -> Html<&'static str> {
        let page_content = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>WebSocket Test</title>
    </head>
    <body>
        <h1>WebSocket Client</h1>
<p>Open the browser console (F12) and run:</p>
<code>socket.send('Hello, WebSocket!');</code>
        <script>
            let socket = new WebSocket('ws://api/test/ping/ws/');

            socket.onopen = function(event) {
                console.log('WebSocket is open now.');
                socket.send('Hello, WebSocket!');
            };

            socket.onmessage = function(event) {
                console.log('Received message:', event.data);
            };

            socket.onclose = function(event) {
                console.log('WebSocket is closed now.');
            };

            socket.onerror = function(error) {
                console.log('WebSocket error:', error);
            };
        </script>
    </body>
    </html>
    "#;

        Html(page_content)
    }
    route("/", get(handler))
}
