use crate::broadcast;
use axum::extract::ws::CloseFrame;
use axum_typed_websockets::{Message, WebSocket};
use dry_console_dto::websocket::CloseCode;
use serde::{Deserialize, Serialize};
use tracing::*;

pub async fn handle_websocket<T, U>(
    mut socket: WebSocket<T, U>,
    mut shutdown: broadcast::Receiver<()>,
    mut on_message: impl FnMut(Message<U>) -> Option<(bool, CloseCode, String)>,
) where
    T: Serialize + for<'de> Deserialize<'de>,
    U: Serialize + for<'de> Deserialize<'de>,
{
    let mut close_code: Option<CloseCode> = None;
    let mut close_message: Option<String> = None;

    loop {
        tokio::select! {
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(item) => {
                        if let Some((should_close, code, message)) = on_message(item) {
                            close_code = Some(code);
                            close_message = Some(message);
                            if should_close {
                                break;
                            }
                        }
                    },
                    Err(err) => {
                        close_code = Some(CloseCode::InvalidFramePayloadData);
                        close_message = Some("Error parsing message.".to_string());
                        error!("Got error: {}", err);
                        break;
                    }
                }
            }
            _ = shutdown.recv() => {
                close_code = Some(CloseCode::GoingAway);
                close_message = Some("Server is shutting down.".to_string());
                break;
            }
        }

        // Check if the loop should break due to shutdown
        if close_code.is_some() {
            break;
        }
    }

    // Disconnect
    let close_frame = CloseFrame {
        code: close_code.unwrap_or(CloseCode::NormalClosure).into(),
        reason: close_message
            .unwrap_or_else(|| "Goodbye.".to_string())
            .into(),
    };
    if let Err(err) = socket.send(Message::Close(Some(close_frame))).await {
        eprintln!("Error sending close frame: {:?}", err);
    }

    if let Err(err) = socket.close().await {
        eprintln!("Error closing the socket: {:?}", err);
    }
    info!("Closed websocket.");
}
