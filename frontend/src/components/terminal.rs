//use patternfly_yew::prelude::*;
use yew::prelude::*;

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::WebSocket;

pub struct TerminalOutputProps {
    messages: Vec<String>,
    ws: Option<WebSocket>,
    callback: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

pub enum Msg {
    NewMessage(String),
    ConnectWebSocket,
}

#[function_component(TerminalOutput)]
pub fn terminal_output() -> Html {
    let messages = use_state_eq(|| Vec::new());
    let ws = use_state(|| None::<WebSocket>);
    let callback = use_state(|| None::<Closure<dyn FnMut(web_sys::MessageEvent)>>);

    {
        let messages = messages.clone();

        use_effect(move || {
            let ws_instance = WebSocket::new("/api/workstation/command_execute").unwrap();

            let cb = {
                let messages = messages.clone();
                Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                    if let Some(text) = e.data().as_string() {
                        // Clone the current state, modify it, and set it
                        messages.set({
                            let mut new_messages = (*messages).clone();
                            new_messages.push(text);
                            new_messages
                        });
                    }
                }) as Box<dyn FnMut(_)>)
            };

            ws_instance.set_onmessage(Some(cb.as_ref().unchecked_ref()));
            ws.set(Some(ws_instance));
            callback.set(Some(cb));

            // Cleanup
            move || {
                if let Some(ws) = (*ws).clone() {
                    ws.close().ok();
                }
            }
        });
    }

    html! {
        <div>
            { for messages.iter().map(|message| html!{ <p>{message}</p> }) }
        </div>
    }
}
