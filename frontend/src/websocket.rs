use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{FileReader, WebSocket};
use yew::prelude::*;

use dry_console_dto::websocket::ServerMsg;

//use crate::random::generate_random_string;

pub fn setup_websocket(
    url: &str,
    on_message: Callback<ServerMsg>,
) -> (
    Rc<RefCell<WebSocket>>,
    Closure<dyn FnMut(web_sys::MessageEvent)>,
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
                        if let Ok(server_msg) = serde_json::from_str::<ServerMsg>(&text) {
                            match server_msg {
                                ServerMsg::Ping => {
                                    ws_clone_inner.borrow().send_with_str("\"Pong\"").unwrap();
                                }
                                ServerMsg::PingReport(r) => {
                                    //let m = r.duration.as_millis();
                                    // log::debug!("{}", format!(
                                    //     "Ping time: {}ms -                                      #{}",
                                    //     m.to_string(),
                                    //     generate_random_string(5)
                                    // ));
                                }
                                _ => {
                                    // Call the custom callback for other messages
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
                onloadend.forget();
            }
        }) as Box<dyn FnMut(_)>)
    };

    ws_instance
        .borrow()
        .set_onmessage(Some(cb.as_ref().unchecked_ref()));

    (ws_instance, cb) // Return the WebSocket and the closure
}
