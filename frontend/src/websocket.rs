use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{FileReader, WebSocket};
use yew::prelude::*;

use dry_console_dto::websocket::ServerMsg;

pub struct WebSocketSetup {
    pub socket: Rc<RefCell<WebSocket>>,
    pub on_message_closure: Rc<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

pub fn setup_websocket(url: &str, on_message: Callback<ServerMsg>) -> WebSocketSetup {
    let ws_instance = Rc::new(RefCell::new(WebSocket::new(url).unwrap()));
    let ws_clone = ws_instance.clone();

    let cb = Rc::new({
        let on_message = on_message.clone();

        Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
            if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let on_message = on_message.clone();
                let ws_clone_inner = ws_clone.clone();

                let onloadend = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
                    if let Some(text) = reader_clone.result().unwrap().as_string() {
                        if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&text) {
                            match server_msg {
                                ServerMsg::Ping => {
                                    ws_clone_inner.borrow().send_with_str("\"Pong\"").unwrap();
                                }
                                _ => {
                                    on_message.emit(server_msg);
                                }
                            }
                        } else {
                            gloo::console::info!("Failed to deserialize ServerMsg :: ", text);
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                reader.set_onloadend(Some(onloadend.as_ref().unchecked_ref()));
                reader.read_as_text(&blob).unwrap();
                onloadend.forget(); // Important: prevent the closure from being dropped
            }
        }) as Box<dyn FnMut(_)>)
    });

    ws_instance
        .borrow()
        .set_onmessage(Some(cb.as_ref().as_ref().unchecked_ref()));

    // Handle WebSocket close event to prevent reconnection
    let on_close = Closure::wrap(Box::new(move |_e: web_sys::CloseEvent| {
        gloo::console::info!("WebSocket closed");
    }) as Box<dyn FnMut(_)>);
    ws_instance
        .borrow()
        .set_onclose(Some(on_close.as_ref().unchecked_ref()));
    on_close.forget(); // Important: prevent the closure from being dropped

    WebSocketSetup {
        socket: ws_instance,
        on_message_closure: cb, // The original Rc<Closure> is kept intact
    }
}
