use gloo::console::debug;
use serde_json::from_str;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{FileReader, WebSocket};
use yew::prelude::*;

use dry_console_dto::websocket::ServerMsg;

pub fn setup_websocket(
    url: &str,
    ws: UseStateHandle<Option<Rc<RefCell<WebSocket>>>>,
    callback: UseStateHandle<Option<Closure<dyn FnMut(web_sys::MessageEvent)>>>,
    on_message: Callback<ServerMsg>,
) {
    let ws_instance = Rc::new(RefCell::new(WebSocket::new(url).unwrap()));
    let ws_clone = ws_instance.clone();

    let cb = {
        let on_message = on_message.clone(); // Clone the callback to avoid moving it into the closure

        Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let on_message = on_message.clone(); // Clone again here to move it inside the inner closure
                let ws_clone_inner = ws_clone.clone(); // Clone WebSocket reference for use in the inner closure

                let onloadend = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
                    if let Some(text) = reader_clone.result().unwrap().as_string() {
                        if let Ok(server_msg) = from_str::<ServerMsg>(&text) {
                            match server_msg {
                                ServerMsg::Ping => {
                                    debug!("Received Ping!");
                                    ws_clone_inner.borrow().send_with_str("\"Pong\"").unwrap();
                                    debug!("Sent Pong!");
                                }
                                _ => {
                                    // Call the custom callback for other messages
                                    on_message.emit(server_msg);
                                }
                            }
                        } else {
                            gloo::console::info!("Failed to deserialize ServerMsg");
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
                reader.read_as_text(&blob).unwrap();
                onloadend.forget();
            }
        }) as Box<dyn FnMut(_)>)
    };

    ws_instance
        .borrow()
        .set_onmessage(Some(cb.as_ref().unchecked_ref()));
    ws.set(Some(ws_instance.clone())); // Store the Rc<RefCell<WebSocket>> directly in the state
    callback.set(Some(cb)); // Update the state
}
