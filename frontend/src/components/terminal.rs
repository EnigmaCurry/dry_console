use crate::{app::WindowDimensions, pages::workstation::WorkstationTab};
use dry_console_dto::websocket::ServerMsg;
use dry_console_dto::websocket::StreamType;
use gloo::console::debug;
use gloo::console::error;
use patternfly_yew::prelude::*;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::MessageEvent;
use web_sys::{HtmlElement, WebSocket};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
    pub show_gutter: bool,
}

enum MsgAction {
    AddMessage { stream: StreamType, message: String },
    Reset,
}

struct MessagesState {
    messages: Vec<(StreamType, String)>,
}

impl Reducible for MessagesState {
    type Action = MsgAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            MsgAction::Reset => MessagesState {
                messages: Vec::new(),
            }
            .into(),
            MsgAction::AddMessage { stream, message } => {
                let mut messages = self.messages.clone();
                messages.push((stream, message));
                MessagesState { messages }.into()
            }
        }
    }
}

pub fn scroll_to_line(node_ref: &NodeRef, line_number: i32) {
    if let Some(element) = node_ref.cast::<web_sys::HtmlElement>() {
        // Calculate the scroll position based on line height and line number
        let line_height = 20; // Adjust this according to your CSS
        let scroll_position = if line_number <= 0 {
            0
        } else if line_number == i32::MAX {
            element.scroll_height()
        } else {
            line_number * line_height
        };

        element.set_scroll_top(scroll_position);
    }
}

#[derive(Debug)]
struct WebSocketState {
    websocket: Option<WebSocket>,
    status: TerminalStatus,
    messages: Vec<(StreamType, String)>,
}
// Reducer actions to manage WebSocketState
#[derive(Debug)]
enum WebSocketAction {
    Initialize,
    Connecting(WebSocket),
    Connected,
    SendMessage(String),
    ReceiveMessage(StreamType, String),
    Processing,
    Complete,
    Failed(String),
    Reset,
}
impl Reducible for WebSocketState {
    type Action = WebSocketAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        debug!(format!("Reducer called with action: {:?}", action));
        debug!(format!("Current state before action: {:?}", *self));

        let new_state: Rc<WebSocketState> = match action {
            WebSocketAction::Initialize => {
                debug!("Action: Initialize");
                WebSocketState {
                    websocket: None,
                    status: TerminalStatus::Initialized,
                    messages: Vec::new(),
                }
                .into()
            }
            WebSocketAction::Connecting(ws) => {
                debug!("Action: Connecting");
                WebSocketState {
                    websocket: Some(ws),
                    status: TerminalStatus::Connecting,
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::Connected => {
                debug!("Action: Connected");
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: TerminalStatus::Ready,
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::SendMessage(message) => {
                debug!("Action: SendMessage, message: {}", message.clone());
                if let Some(ws) = &self.websocket {
                    ws.send_with_str(&message).ok();
                }
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: self.status.clone(),
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::ReceiveMessage(stream, message) => {
                debug!(format!(
                    "Action: ReceiveMessage, stream: {:?}, message: {}",
                    stream, message
                ));
                let mut messages = self.messages.clone();
                messages.push((stream, message));
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: if self.status == TerminalStatus::Connecting {
                        TerminalStatus::Ready
                    } else {
                        self.status.clone()
                    },
                    messages,
                }
                .into()
            }
            WebSocketAction::Processing => {
                debug!("Action: Processing");
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: TerminalStatus::Processing,
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::Complete => {
                debug!("Action: Complete");
                WebSocketState {
                    websocket: None,
                    status: TerminalStatus::Complete,
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::Failed(error_message) => {
                debug!("Action: Failed, error_message: {}", error_message.clone());
                let mut messages = self.messages.clone();
                messages.push((
                    StreamType::Meta,
                    format!("# [Process failed]: {}", error_message),
                ));
                WebSocketState {
                    websocket: None,
                    status: TerminalStatus::Failed,
                    messages,
                }
                .into()
            }
            WebSocketAction::Reset => {
                debug!("Action: Reset");
                WebSocketState {
                    websocket: None,
                    status: TerminalStatus::Initialized,
                    messages: Vec::new(),
                }
                .into()
            }
        };

        debug!(format!("New state after action: {:?}", *new_state));
        new_state
    }
}

#[derive(PartialEq, Debug, Clone)]
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
    let screen_dimensions = use_context::<WindowDimensions>().expect("no ctx found");
    let messages = use_reducer(|| MessagesState {
        messages: Vec::new(),
    });
    let callback_state = use_state(|| None::<Callback<MouseEvent>>);
    let status = use_state(|| TerminalStatus::Initialized);
    let num_lines = use_state(|| 1);

    let terminal_ref = use_node_ref();
    let gutter_ref = use_node_ref();

    let ws_state = use_reducer(|| WebSocketState {
        websocket: None,
        status: TerminalStatus::Initialized,
        messages: Vec::new(),
    });

    // Cleanup websocket on tab change
    {
        let status = status.clone();
        let ws_state = ws_state.clone();
        let callback_state = callback_state.clone();
        let messages = messages.clone();

        use_effect_with(props.selected_tab.clone(), move |_| {
            move || {
                if *status != TerminalStatus::Initialized {
                    if let Some(ws) = &ws_state.websocket {
                        ws.close().ok(); // Close the WebSocket
                    }
                    callback_state.set(None);
                    if *status != TerminalStatus::Complete && *status != TerminalStatus::Failed {
                        status.set(TerminalStatus::Failed);
                        messages.dispatch(MsgAction::AddMessage {
                            stream: StreamType::Meta,
                            message: "# [Process failed]".to_string(),
                        });
                    }
                }
            }
        });
    }

    // Update gutter height dynamically
    {
        let messages_len = messages.messages.len();
        let screen_dimensions = screen_dimensions.clone();
        let num_lines = num_lines.clone();
        use_effect_with(
            [messages.messages.len(), screen_dimensions.height as usize],
            move |_| {
                if screen_dimensions.height < 900.0 {
                    if messages_len > 13 {
                        num_lines.set(13);
                    } else if messages_len > 0 {
                        num_lines.set(messages_len);
                    }
                } else {
                    if messages_len > 24 {
                        num_lines.set(24);
                    } else if messages_len > 0 {
                        num_lines.set(messages_len);
                    }
                }
                || ()
            },
        );
    }

    // Sync the scroll of the gutter to the output:
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
    let scroll_to_top = {
        let terminal_ref = terminal_ref.clone();
        Callback::from(move |_: MouseEvent| {
            scroll_to_line(&terminal_ref, 0);
        })
    };
    let scroll_to_bottom = {
        let terminal_ref = terminal_ref.clone();
        Callback::from(move |_: MouseEvent| {
            scroll_to_line(&terminal_ref, i32::MAX);
        })
    };
    // Effect to scroll to the bottom on first render
    {
        let terminal_ref = terminal_ref.clone();
        use_effect(move || {
            scroll_to_line(&terminal_ref, i32::MAX);
            || ()
        });
    }

    // "Run command" button callback to set up WebSocket and change status

    // Reset reinitializes websocket and terminal
    let reset_terminal = {
        let ws_state = ws_state.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(ws) = &ws_state.websocket {
                ws.send_with_str("\"Cancel\"").ok();
                ws.close().ok();
            }
            ws_state.dispatch(WebSocketAction::Reset);
        })
    };
    let run_command = {
        let ws_state = ws_state.clone();
        Callback::from(move |_: MouseEvent| {
            // Close any existing WebSocket before starting a new one
            if let Some(ws) = &ws_state.websocket {
                ws.close().ok();
            }
            ws_state.dispatch(WebSocketAction::Reset);

            // Attempt to connect the WebSocket
            if let Ok(ws) = WebSocket::new("/api/workstation/command_execute/") {
                let ws_clone = ws.clone();
                ws_state.dispatch(WebSocketAction::Connecting(ws));

                let onopen_callback = {
                    let ws_state = ws_state.clone();
                    Closure::wrap(Box::new(move |_| {
                        ws_state.dispatch(WebSocketAction::Connected);
                    }) as Box<dyn FnMut(JsValue)>)
                };
                ws_clone.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
                onopen_callback.forget();

                let onmessage_callback = {
                    let ws_state = ws_state.clone();
                    Closure::wrap(Box::new(move |event: MessageEvent| {
                        if let Some(msg) = event.data().as_string() {
                            // Handle incoming messages and update state
                            if msg.contains("PingReport") {
                                ws_state.dispatch(WebSocketAction::ReceiveMessage(
                                    StreamType::Meta,
                                    msg,
                                ));
                            } else if msg.contains("Complete") {
                                ws_state.dispatch(WebSocketAction::Complete);
                            } else if msg.contains("Failed") {
                                ws_state.dispatch(WebSocketAction::Failed(msg));
                            } else {
                                ws_state.dispatch(WebSocketAction::ReceiveMessage(
                                    StreamType::Stdout,
                                    msg,
                                ));
                            }
                        }
                    }) as Box<dyn FnMut(MessageEvent)>)
                };
                ws_clone.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();

                let onerror_callback = {
                    let ws_state = ws_state.clone();
                    Closure::wrap(Box::new(move |error: ErrorEvent| {
                        ws_state
                            .dispatch(WebSocketAction::Failed(format!("{:?}", error.message())));
                    }) as Box<dyn FnMut(ErrorEvent)>)
                };
                ws_clone.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();
            }
        })
    };

    let mut line_number_gutter = 1;
    let mut line_number_output = 1;
    let should_show_gutter = *status != TerminalStatus::Initialized && props.show_gutter;

    html! {
        <div class="terminal">
            <div class="toolbar pf-u-display-flex pf-u-justify-content-space-between">
                <div class="pf-u-display-flex">
                    <Button onclick={run_command.clone()}>{"🚀 Run command"}</Button>
                    <Button onclick={reset_terminal.clone()}>{"💥 Reset"}</Button>
                </div>
                <div class="pf-u-display-flex">
                    <Button onclick={scroll_to_top.clone()}>{"⬆️ Top"}</Button>
                    <Button onclick={scroll_to_bottom.clone()}>{"⬇️ Bottom"}</Button>
                </div>
            </div>
            <div class="content">
                if should_show_gutter {
                    <div class="gutter" ref={gutter_ref} style={format!("max-height: {}em", *num_lines + 1)}>
                        {
                            for ws_state.messages.iter().map(|(stream, _message)| {
                                let gutter_content = match stream {
                                    StreamType::Stdout => {
                                        let content = line_number_gutter.to_string();
                                        line_number_gutter += 1;
                                        content
                                    }
                                    StreamType::Stderr => "E".to_string(),         // "E" for StdErr
                                    StreamType::Meta => "#".to_string(),           // "M" for Meta
                                };
                                html!{
                                    <div class="gutter-line">{gutter_content}</div>
                                }
                            })
                        }
                    </div>
                }

                <div class="output" ref={terminal_ref} {onscroll} style={format!("max-height: {}em", *num_lines + 1)}>
                    {
                        for ws_state.messages.iter().map(|(stream, message)| {
                            let class_name = match stream {
                                StreamType::Stdout => "stream-stdout",
                                StreamType::Stderr => "stream-stderr",
                                StreamType::Meta => "stream-meta",
                            };
                            let id = if *stream == StreamType::Stdout {
                                let id = format!("line-{}", line_number_output);
                                line_number_output += 1;
                                id
                            } else {
                                "".to_string()
                            };
                            html!{
                                <p id={id} class={class_name}>{message}</p>
                            }
                        })
                    }
                </div>
            </div>
        </div>
    }
}
