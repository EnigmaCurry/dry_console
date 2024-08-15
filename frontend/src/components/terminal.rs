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

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let messages = use_state_eq(Vec::new);
    let ws = use_state(|| None::<Rc<RefCell<WebSocket>>>); // Update the type here
    let callback = use_state(|| None::<Closure<dyn FnMut(web_sys::MessageEvent)>>);
    let is_connected = use_state(|| false); // Track connection status

    {
        let messages = messages.clone();
        let selected_tab = props.selected_tab.clone();
        let ws_clone = ws.clone();
        let callback_clone = callback.clone();
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

                setup_websocket(
                    "/api/workstation/command_execute/",
                    ws_clone.clone(),
                    callback_clone.clone(),
                    on_message,
                );

                is_connected_clone.set(true); // Mark as connected
            }

            // Cleanup function: close WebSocket if tab changes and WebSocket is connected
            move || {
                if selected_tab != WorkstationTab::DRymcgTech && *is_connected_clone {
                    if let Some(ws_instance) = (*ws_clone).as_ref() {
                        ws_instance.borrow().close().ok();
                    }
                    ws_clone.set(None);
                    callback_clone.set(None);
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
