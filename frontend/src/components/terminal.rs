use crate::random::generate_random_string;
use crate::{pages::workstation::WorkstationTab, websocket::setup_websocket};
use dry_console_dto::websocket::ServerMsg;
use gloo::console::debug;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use web_sys::WebSocket;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

enum MsgAction {
    AddMessage(String),
}

struct MessagesState {
    messages: Vec<String>,
}

impl Reducible for MessagesState {
    type Action = MsgAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MsgAction::AddMessage(new_message) => {
                let mut messages = self.messages.clone();
                messages.push(new_message);
                MessagesState { messages }.into()
            }
        }
    }
}

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let messages = use_reducer(|| MessagesState {
        messages: Vec::new(),
    });
    let ws_state = use_state(|| None);
    let callback_state = use_state(|| None);
    let is_connected = use_state(|| false); // Track connection status

    {
        let selected_tab = props.selected_tab.clone();
        let ws_state_clone = ws_state.clone();
        let callback_state_clone = callback_state.clone();
        let is_connected_clone = is_connected.clone();
        let messages_clone = messages.clone();

        use_effect_with(selected_tab, move |selected_tab| {
            if *selected_tab == WorkstationTab::DRymcgTech && !*is_connected_clone {
                let messages_ref = messages_clone.clone();
                let on_message = Callback::from(move |server_msg: ServerMsg| {
                    let new_message = format!("{:?} {}", server_msg, generate_random_string(5));
                    messages_ref.dispatch(MsgAction::AddMessage(new_message));
                });

                debug!("setup_websocket");
                let setup = setup_websocket("/api/workstation/command_execute/", on_message);

                ws_state_clone.set(Some(Rc::new(RefCell::new(setup.socket))));
                callback_state_clone.set(Some(setup.on_message_closure));
                is_connected_clone.set(true); // Mark as connected
            }

            move || {
                if *is_connected_clone {
                    if let Some(ws_rc) = &*ws_state_clone {
                        let ws = ws_rc.borrow();
                        ws.borrow().close().ok(); // Properly close the WebSocket
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
            { for messages.messages.iter().map(|message| html!{ <p>{message}</p> }) }
        </div>
    }
}
