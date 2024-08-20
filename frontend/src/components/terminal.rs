use crate::{
    app::WindowDimensions, pages::workstation::WorkstationTab, websocket::setup_websocket,
};
use dry_console_dto::websocket::ServerMsg;
use dry_console_dto::websocket::StreamType;
use gloo::console::debug;
use patternfly_yew::prelude::*;
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
    let screen_dimensions = use_context::<WindowDimensions>().expect("no ctx found");
    let messages = use_reducer(|| MessagesState {
        messages: Vec::new(),
    });
    let ws_state: UseStateHandle<Rc<RefCell<Option<WebSocket>>>> =
        use_state(|| Rc::new(RefCell::new(None)));
    let callback_state = use_state(|| None);
    let status = use_state(|| TerminalStatus::Initialized);
    let num_lines = use_state(|| 1);

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
                                messages_clone.dispatch(MsgAction::AddMessage {
                                    stream: StreamType::Meta,
                                    message: "# [Ready]".to_string(),
                                });
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
                            messages_clone.dispatch(MsgAction::AddMessage {
                                stream: msg.stream,
                                message: msg.line,
                            });
                        }
                        ServerMsg::ProcessComplete(msg) => match msg.code {
                            0 => {
                                status_clone.set(TerminalStatus::Complete);
                                messages_clone.dispatch(MsgAction::AddMessage {
                                    stream: StreamType::Meta,
                                    message: "# [Process complete]".to_string(),
                                });
                            }
                            _ => {
                                status_clone.set(TerminalStatus::Failed);
                                messages_clone.dispatch(MsgAction::AddMessage {
                                    stream: StreamType::Meta,
                                    message: "# [Process failed]".to_string(),
                                });
                            }
                        },
                    };
                });

                //Setup websocket:
                status.set(TerminalStatus::Connecting);
                let setup = setup_websocket("/api/workstation/command_execute/", on_message);
                *ws_state.borrow_mut() = Some(setup.socket.borrow().clone()); // Unwrap and clone the WebSocket
                callback_state.set(Some(setup.on_message_closure));
                messages.dispatch(MsgAction::AddMessage {
                    stream: StreamType::Meta,
                    message: "# [Connecting...]".to_string(),
                });
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
                        messages.dispatch(MsgAction::AddMessage {
                            stream: StreamType::Meta,
                            message: "# [Process failed]".to_string(),
                        });
                    }
                }
            }
        });
    }

    //Update gutter height dynamically:
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
    let mut line_number = 1;

    let scroll_to_top = {
        let terminal_ref = terminal_ref.clone();
        Callback::from(move |_: MouseEvent| {
            scroll_to_line(&terminal_ref.clone(), 0);
        })
    };

    let scroll_to_bottom = {
        let terminal_ref = terminal_ref.clone();
        Callback::from(move |_: MouseEvent| {
            scroll_to_line(&terminal_ref.clone(), i32::MAX);
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
    html! {
        <div class="terminal">
            <div class="toolbar pf-u-display-flex pf-u-justify-content-space-between">
                <div class="pf-u-display-flex">
                    <Button>{"Run command"}</Button>
                    <Button>{"Reset"}</Button>
                </div>
                <div class="pf-u-display-flex">
                    <Button onclick={scroll_to_top}>{"Top"}</Button>
                    <Button onclick={scroll_to_bottom}>{"Bottom"}</Button>
                </div>
            </div>
            <div class="content">
                if props.show_gutter {
                    <div class="gutter" ref={gutter_ref} style={format!("max-height: {}em", *num_lines + 1)}>
                        {
                            for messages.messages.iter().map(|(stream, _message)| {
                                let gutter_content = match stream {
                                    StreamType::Stdout => {
                                        let content = line_number.to_string();
                                        line_number += 1;
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
                        for messages.messages.iter().map(|(stream, message)| {
                            let class_name = match stream {
                                StreamType::Stdout => "stream-stdout",
                                StreamType::Stderr => "stream-stderr",
                                StreamType::Meta => "stream-meta",
                            };
                            let id = if *stream == StreamType::Stdout {
                                let id = format!("line-{}", line_number);
                                line_number += 1;
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
