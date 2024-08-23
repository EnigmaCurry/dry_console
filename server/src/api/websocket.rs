use crate::broadcast;
use axum::extract::ws::CloseFrame;
use axum_typed_websockets::{Message, WebSocket};
use dry_console_dto::websocket::CloseCode;
use dry_console_dto::websocket::WebSocketMessage;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;

use tracing::*;

const PING_INTERVAL: u64 = 15000;

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
    T: WebSocketMessage + 'static,
    U: WebSocketMessage + 'static + PartialEq,
    F: FnMut(Message<U>) -> Pin<Box<dyn Future<Output = Option<WebSocketResponse>> + Send>>,
{
    let last_ping = Arc::new(Mutex::new(None));
    let mut ping_interval = tokio::time::interval(Duration::from_millis(PING_INTERVAL));
    let mut ping_timeout: Option<Pin<Box<tokio::time::Sleep>>> = None;

    let mut consecutive_missed_pongs = 0;
    let mut close_code: Option<CloseCode>;
    let mut close_message: Option<String>;

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                let mut socket_guard = socket.lock().await;
                if consecutive_missed_pongs > 3 {
                    close_code = Some(CloseCode::PolicyViolation);
                    close_message = Some("Missed 3 consecutive pongs. Disconnecting...".to_string());
                    debug!("{}", close_message.as_ref().unwrap());
                    break;
                }

                *last_ping.lock().await = Some(Instant::now());
                if let Some(socket) = socket_guard.as_mut() {
                    socket.send(Message::Item(T::PING)).await.ok();
                    //debug!("Ping message sent!");
                    ping_timeout = Some(Box::pin(tokio::time::sleep(Duration::from_secs(20))));
                } else {
                    close_code = Some(CloseCode::NormalClosure);
                    close_message = Some("Socket closed".to_string());
                    break;
                }

                consecutive_missed_pongs += 1;
            },
            _ = async {
                if let Some(timeout) = &mut ping_timeout {
                    timeout.await;
                    true
                } else {
                    false
                }
            } => {
                if (*last_ping.lock().await).is_some() {
                    close_code = Some(CloseCode::PolicyViolation);
                    close_message = Some("Pong response not received in time".to_string());
                    debug!("Pong response not received within 20s. Disconnecting...");
                    break;
                }
            },
            msg = async {
                let mut socket_guard = socket.lock().await;
                if let Some(socket) = socket_guard.as_mut() {
                    socket.recv().await
                } else {
                    None
                }
            } => {
                match msg {
                    Some(Ok(Message::Item(msg))) if msg == U::PONG => {
                        //debug!("Pong message received!");
                        let mut last_ping_guard = last_ping.lock().await;
                        if let Some(instant) = *last_ping_guard {
                            *last_ping_guard = None;
                            if let Some(socket) = socket.lock().await.as_mut() {
                                socket.send(Message::Item(T::ping_report(Instant::now().duration_since(instant)))).await.ok();
                            }
                            ping_timeout = None;
                            consecutive_missed_pongs = 0;
                        } else {
                            close_code = Some(CloseCode::UnsupportedData);
                            close_message = Some("Unexpected Pong".to_string());
                            debug!("Unexpected Pong.");
                            break;
                        }
                    },
                    Some(Ok(item)) => {
                        if let Some(response) = on_message(item).await {
                            close_code = Some(response.close_code);
                            close_message = Some(response.close_message);
                            if response.close {
                                debug!("Closing : {:?} {:?}", close_code, close_message);
                                break;
                            }
                        }
                    },
                    Some(Err(err)) => {
                        close_code = Some(CloseCode::InvalidFramePayloadData);
                        close_message = Some(format!("Error parsing message: {}", err));
                        debug!("Websocket closed after parse error: {}",err);
                        break;
                    },
                    None => {
                        close_code = Some(CloseCode::NormalClosure);
                        close_message = Some("Connection closed by client.".to_string());
                        debug!("Websocket closed by client.");
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

    debug!("Disconnecting socket...");
    let close_frame = CloseFrame {
        code: close_code.unwrap_or(CloseCode::NormalClosure).into(),
        reason: close_message
            .unwrap_or_else(|| "Goodbye.".to_string())
            .into(),
    };

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
