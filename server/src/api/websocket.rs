use crate::broadcast;
use axum::extract::ws::CloseFrame;
use axum_typed_websockets::{Message, WebSocket};
use dry_console_dto::websocket::CloseCode;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use tracing::*;

#[derive(Debug)]
pub struct WebSocketResponse {
    pub close: bool,
    pub close_code: CloseCode,
    pub close_message: String,
}

pub async fn handle_websocket<T, U, F>(
    socket: Arc<Mutex<Option<WebSocket<T, U>>>>,
    mut shutdown: broadcast::Receiver<()>,
    mut on_message: F,
) where
    T: Serialize + for<'de> Deserialize<'de> + 'static,
    U: Serialize + for<'de> Deserialize<'de> + 'static,
    F: FnMut(Message<U>) -> Pin<Box<dyn Future<Output = Option<WebSocketResponse>> + Send>>,
{
    let mut close_code: Option<CloseCode> = None;
    let mut close_message: Option<String> = None;

    loop {
        tokio::select! {
            msg = async {
                let mut socket_guard = socket.lock().await;
                if let Some(socket) = socket_guard.as_mut() {
                    socket.recv().await
                } else {
                    None
                }
            } => {
                match msg {
                    Some(Ok(item)) => {
                        if let Some(response) = on_message(item).await {
                            close_code = Some(response.close_code);
                            close_message = Some(response.close_message);
                            if response.close {
                                break;
                            }
                        }
                    },
                    Some(Err(err)) => {
                        close_code = Some(CloseCode::InvalidFramePayloadData);
                        close_message = Some(format!("Error parsing message: {}", err));
                        break;
                    },
                    None => {
                        close_code = Some(CloseCode::NormalClosure);
                        close_message = Some("Connection closed by client.".to_string());
                        break;
                    }
                }
            },
            _ = shutdown.recv() => {
                close_code = Some(CloseCode::GoingAway);
                close_message = Some("Server is shutting down.".to_string());
                break;
            },
        }
    }

    // Disconnect
    debug!("Disconnecting socket...");
    let close_frame = CloseFrame {
        code: close_code.unwrap_or(CloseCode::NormalClosure).into(),
        reason: close_message
            .unwrap_or_else(|| "Goodbye.".to_string())
            .into(),
    };

    // Take ownership of the WebSocket and close it
    let mut socket_guard = socket.lock().await;
    if let Some(mut socket_owned) = socket_guard.take() {
        if let Err(err) = socket_owned.send(Message::Close(Some(close_frame))).await {
            eprintln!("Error sending close frame: {:?}", err);
        }

        if let Err(err) = socket_owned.close().await {
            eprintln!("Error closing the socket: {:?}", err);
        }
    }

    info!("Closed websocket.");
}
