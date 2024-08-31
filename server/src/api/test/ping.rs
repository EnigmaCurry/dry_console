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
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

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

    async fn websocket(mut socket: WebSocket<ServerMsg, ClientMsg>) {
        let start = Instant::now();
        socket.send(Message::Item(ServerMsg::Ping)).await.ok();
        if let Some(msg) = socket.recv().await {
            match msg {
                Ok(Message::Item(ClientMsg::Pong)) => {
                    info!("ping: {:?}", start.elapsed());
                }
                Ok(other) => {
                    warn!("Received unexpected message: {:?}", other);
                }
                Err(err) => {
                    error!("Got error: {}", err);
                }
            }
        }
        info!("Closed websocket.");
    }

    async fn upgrade(ws: WebSocketUpgrade<ServerMsg, ClientMsg>) -> impl IntoResponse {
        debug!("Websocket upgrade request received.");
        ws.on_upgrade(websocket)
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
            let socket = new WebSocket('/api/test/ping/ws/');

            socket.onopen = function(event) {
                console.log('WebSocket is open now.');
            };
            function blobToString(blob) {
                return new Promise((resolve, reject) => {
                    const reader = new FileReader();
                    reader.onloadend = function() {
                        resolve(reader.result);
                    };
                    reader.onerror = reject;
                    reader.readAsText(blob);
                });
            }
            socket.onmessage = function(event) {
                blobToString(event.data).then((msg) => {
                    console.log('Received message:', msg);
                    item = msg.replace(/^"|"$/g, '');
                    if(item === "Ping") {
                        socket.send("\"Pong\"");
                    } else {
                        console.warn("Invalid item:", item);
                    }
                })
                
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
