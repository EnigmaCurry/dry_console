use gloo::console::{debug, info};
//use patternfly_yew::prelude::*;
use crate::{pages::workstation::WorkstationTab, websocket::setup_websocket};
use dry_console_dto::websocket::ServerMsg;
use serde_json::from_str;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{FileReader, WebSocket};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

pub struct TerminalOutputState {
    ws: Option<Rc<RefCell<WebSocket>>>,
    _callback: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

impl Drop for TerminalOutputState {
    fn drop(&mut self) {
        debug!("Drop!");
        if let Some(ws) = &self.ws {
            ws.borrow().close().ok(); // Properly close the WebSocket
        }
    }
}

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let messages = use_state_eq(Vec::new);
    let ws_state = use_state(|| None);
    let callback_state = use_state(|| None);
    let is_connected = use_state(|| false); // Track connection status

    {
        let messages = messages.clone();
        let selected_tab = props.selected_tab.clone();
        let ws_state_clone = ws_state.clone();
        let callback_state_clone = callback_state.clone();
        let is_connected_clone = is_connected.clone();

        use_effect(move || {
            if selected_tab == WorkstationTab::DRymcgTech && !*is_connected_clone {
                let on_message = Callback::from(move |server_msg: ServerMsg| {
                    messages.set({
                        let mut new_messages = (*messages).clone();
                        new_messages.push(format!("{:?}", server_msg));
                        new_messages
                    });
                });

                let (ws, callback) =
                    setup_websocket("/api/workstation/command_execute/", on_message);

                ws_state_clone.set(Some(Rc::new(RefCell::new(ws))));
                callback_state_clone.set(Some(callback));
                is_connected_clone.set(true); // Mark as connected
            }

            move || {
                if selected_tab != WorkstationTab::DRymcgTech && *is_connected_clone {
                    if let Some(ws_rc) = &*ws_state_clone {
                        // Borrow the RefCell, then borrow the WebSocket inside it
                        let ws = ws_rc.borrow();
                        ws.borrow().close().ok(); // Close the WebSocket
                    }
                    ws_state_clone.set(None);
                    callback_state_clone.set(None);
                    is_connected_clone.set(false); // Reset connection status
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
