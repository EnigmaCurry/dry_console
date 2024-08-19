use crate::{pages::workstation::WorkstationTab, websocket::setup_websocket};
use dry_console_dto::websocket::ServerMsg;
use gloo::console::debug;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, WebSocket};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
    pub show_gutter: bool,
}

enum MsgAction {
    AddMessage(String),
    Reset,
}

struct MessagesState {
    messages: Vec<String>,
}

impl Reducible for MessagesState {
    type Action = MsgAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MsgAction::Reset => {
                let mut messages = self.messages.clone();
                messages.clear();
                MessagesState { messages }.into()
            }
            MsgAction::AddMessage(new_message) => {
                let mut messages = self.messages.clone();
                messages.push(new_message);
                MessagesState { messages }.into()
            }
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn scroll_to_line(container_id: &str, mut line_number: i32) {
    let window = web_sys::window().expect("should have a Window");
    let document = window.document().expect("should have a Document");

    if let Some(container_element) = document.get_element_by_id(container_id) {
        let children = container_element.children();
        let total_lines = children.length() as i32;
        if line_number < 1 {
            line_number = 1;
        } else if line_number >= 2 {
            line_number -= 1;
            if line_number >= total_lines {
                line_number = total_lines;
            }
        }

        let line_id = format!("line-{}", line_number);

        if let Some(line_element) = document.get_element_by_id(&line_id) {
            // Scroll the container such that the line is visible
            line_element.scroll_into_view_with_bool(true);
        } else {
            log(&format!("Line element with id '{}' not found", line_id));
        }
    } else {
        log(&format!(
            "Container element with id '{}' not found",
            container_id
        ));
    }
}

#[derive(PartialEq, Debug)]
enum TerminalStatus {
    Initialized,
    Connecting,
    Ready,
    Processing,
    Failed,
    Complete,
}

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let messages = use_reducer(|| MessagesState {
        messages: Vec::new(),
    });
    let ws_state: UseStateHandle<Rc<RefCell<Option<WebSocket>>>> =
        use_state(|| Rc::new(RefCell::new(None)));
    let callback_state = use_state(|| None);
    let status = use_state(|| TerminalStatus::Initialized);

    // NodeRefs for terminal and gutter
    let terminal_ref = use_node_ref();
    let gutter_ref = use_node_ref();

    {
        let selected_tab = props.selected_tab.clone();
        let ws_state = ws_state.clone();
        let callback_state = callback_state.clone();
        let messages = messages.clone();
        let status_clone = status.clone();

        use_effect_with(selected_tab, move |selected_tab| {
            if *selected_tab == WorkstationTab::DRymcgTech
                && *status_clone == TerminalStatus::Initialized
            {
                let messages_clone = messages.clone();
                let ws_state_clone = ws_state.clone();
                let on_message = Callback::from(move |server_msg: ServerMsg| {
                    match server_msg {
                        ServerMsg::Ping | ServerMsg::Pong {} => {}
                        ServerMsg::PingReport(_r) => {
                            if *status_clone == TerminalStatus::Initialized
                                || *status_clone == TerminalStatus::Connecting
                            {
                                status_clone.set(TerminalStatus::Ready);
                                messages_clone.dispatch(MsgAction::Reset);
                                messages_clone
                                    .dispatch(MsgAction::AddMessage("# [Ready]".to_string()));
                                if let Some(ws) = &*ws_state_clone.borrow() {
                                    ws.send_with_str(
                                        "{\"Command\": { \"id\": \"01J5NN55HAWZJS96BJMHQG4XJD\"}}",
                                    )
                                    .unwrap();
                                }
                            }
                        }
                        ServerMsg::Process(_process) => {
                            status_clone.set(TerminalStatus::Processing);
                            messages_clone.dispatch(MsgAction::Reset);
                        }
                        ServerMsg::ProcessOutput(msg) => {
                            status_clone.set(TerminalStatus::Processing);
                            messages_clone.dispatch(MsgAction::AddMessage(msg.line));
                        }
                        ServerMsg::ProcessComplete(msg) => match msg.code {
                            0 => {
                                status_clone.set(TerminalStatus::Complete);
                                messages_clone.dispatch(MsgAction::AddMessage(
                                    "# [Process complete]".to_string(),
                                ));
                            }
                            _ => {
                                status_clone.set(TerminalStatus::Failed);
                                messages_clone.dispatch(MsgAction::AddMessage(
                                    "# [Process failed]".to_string(),
                                ));
                            }
                        },
                    };
                });

                debug!("setup_websocket");
                status.set(TerminalStatus::Connecting);
                let setup = setup_websocket("/api/workstation/command_execute/", on_message);
                *ws_state.borrow_mut() = Some(setup.socket.borrow().clone()); // Unwrap and clone the WebSocket
                callback_state.set(Some(setup.on_message_closure));
                messages.dispatch(MsgAction::AddMessage("# [Connecting...]".to_string()));
            }

            move || {
                if *status != TerminalStatus::Initialized {
                    if let Some(ws) = &*ws_state.borrow() {
                        ws.close().ok(); // Close the WebSocket
                    }
                    *ws_state.borrow_mut() = None; // Set WebSocket to None
                    callback_state.set(None);
                    if *status != TerminalStatus::Complete && *status != TerminalStatus::Failed {
                        status.set(TerminalStatus::Failed);
                        messages.dispatch(MsgAction::AddMessage("# [Process failed]".to_string()));
                    }
                }
            }
        });
    }

    let onscroll = {
        let terminal_ref = terminal_ref.clone();
        let gutter_ref = gutter_ref.clone();
        Callback::from(move |_| {
            if let (Some(terminal), Some(gutter)) = (
                terminal_ref.cast::<HtmlElement>(),
                gutter_ref.cast::<HtmlElement>(),
            ) {
                let scroll_top = terminal.scroll_top();
                gutter.set_scroll_top(scroll_top);
            }
        })
    };

    html! {
        <div class="terminal">
            if props.show_gutter {
                <div class="gutter" ref={gutter_ref}>
                    { for (1..=messages.messages.len()).map(|line_number| html!{ <div class="gutter-line">{line_number}</div> }) }
                </div>
            }
            <div class="output" ref={terminal_ref} {onscroll}>
                { for messages.messages.iter().enumerate().map(|(index, message)| html!{ <p id={format!("line-{}", index + 1)}>{message}</p> }) }
            </div>
        </div>
    }
}
