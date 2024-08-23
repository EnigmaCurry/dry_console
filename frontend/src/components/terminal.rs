use crate::{app::WindowDimensions, pages::workstation::WorkstationTab};
use dry_console_dto::websocket::Command;
use dry_console_dto::websocket::PingReport;
use dry_console_dto::websocket::ServerMsg;
use dry_console_dto::websocket::StreamType;
use gloo::console::debug;
use gloo::console::error;
use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use patternfly_yew::prelude::*;
use serde_json::from_str;
use std::rc::Rc;
use ulid::Ulid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::js_sys;
use web_sys::Blob;
use web_sys::Element;
use web_sys::FileReader;
use web_sys::HtmlInputElement;
use web_sys::MessageEvent;
use web_sys::Window;
use web_sys::{HtmlElement, WebSocket};
use yew::prelude::*;

const SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY: &str = "terminal:show_line_numbers";
const BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY: &str = "terminal:background_color_change";
const BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY: &str = "terminal:background_color_success";
const BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY: &str = "terminal:background_color_failure";
const BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY: &str = "terminal:background_color_normal";

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
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
    Connect(WebSocket),
    ReceivePingReport(PingReport),
    ReceiveProcessOutput(StreamType, String),
    ReceiveProcessComplete(String, usize),
    ReceiveProcess(Ulid),
    Failed(String),
    Reset,
    SendPong,
}
impl Reducible for WebSocketState {
    type Action = WebSocketAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        //debug!(format!("Reducer called with action: {:?}", action));
        //debug!(format!("Current state before action: {:?}", *self));

        let new_state: Rc<WebSocketState> = match action {
            WebSocketAction::Connect(ws) => {
                //debug!("Action: Connect");
                WebSocketState {
                    websocket: Some(ws),
                    status: TerminalStatus::Connecting,
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::ReceivePingReport(_r) => {
                //debug!(format!("Action: ReceivePingReport {:?}", r));
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: if self.status == TerminalStatus::Connecting {
                        if let Some(ws) = &self.websocket {
                            if let Ok(serialized_msg) =
                                serde_json::to_string(&Command { id: Ulid::new() })
                            {
                                //debug!(format!("sending command serialized: {}", serialized_msg));
                                ws.send_with_str(&serialized_msg).ok();
                            }
                        }
                        TerminalStatus::Ready
                    } else {
                        self.status.clone()
                    },
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::ReceiveProcess(_id) => {
                //debug!(format!("Action: ReceiveProcess, id: {:?}", id));
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: TerminalStatus::Processing,
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::ReceiveProcessOutput(stream, message) => {
                //debug!(format!(
                //    "Action: ReceiveProcessOutput, stream: {:?}, message: {}",
                //    stream, message
                //));
                let mut messages = self.messages.clone();
                messages.push((stream, message));
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: self.status.clone(),
                    messages,
                }
                .into()
            }
            WebSocketAction::ReceiveProcessComplete(_id, code) => {
                //debug!(format!(
                //                    "Action: ReceiveProcessComplete, id: {:?}, code: {}",
                //    id, code
                //));
                WebSocketState {
                    websocket: self.websocket.clone(),
                    status: if code == 0 {
                        TerminalStatus::Complete
                    } else {
                        TerminalStatus::Failed
                    },
                    messages: self.messages.clone(),
                }
                .into()
            }
            WebSocketAction::Failed(error_message) => {
                //debug!("Action: Failed, error_message: {}", error_message.clone());
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
                //debug!("Action: Reset");
                if let Some(ws) = &self.websocket {
                    debug!("Closing socket");
                    ws.close().ok();
                }
                WebSocketState {
                    websocket: None,
                    status: TerminalStatus::Initialized,
                    messages: Vec::new(),
                }
                .into()
            }
            WebSocketAction::SendPong => {
                //debug!("Action: SendPong");
                if let Some(ws) = &self.websocket {
                    let msg = ServerMsg::Pong;
                    if let Ok(serialized_msg) = serde_json::to_string(&msg) {
                        ws.send_with_str(&serialized_msg).ok();
                    } else {
                        error!("Failed to serialize ServerMsg::Pong");
                    }
                }
                self.clone()
            }
        };

        //debug!(format!("New state after action: {:?}", *new_state));
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
pub fn terminal_output(_props: &TerminalOutputProps) -> Html {
    let screen_dimensions = use_context::<WindowDimensions>().expect("no ctx found");
    let num_lines = use_state(|| 1);
    let show_line_numbers =
        use_state(|| LocalStorage::get::<bool>(SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY).unwrap_or(true));
    let background_color_change = use_state(|| {
        LocalStorage::get::<bool>(BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY).unwrap_or(true)
    });
    let background_color_success = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY)
            .unwrap_or("#416d58".to_string())
    });
    let background_color_success_clone = background_color_success.clone();
    let background_color_failure = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY)
            .unwrap_or("#a60d31".to_string())
    });
    let background_color_failure_clone = background_color_failure.clone();
    let background_color_normal = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY)
            .unwrap_or("#000".to_string())
    });
    let background_color_failure_clone = background_color_failure.clone();
    let user_attempted_scroll = use_state(|| false);
    let terminal_ref = use_node_ref();
    let gutter_ref = use_node_ref();

    let ws_state = use_reducer(|| WebSocketState {
        websocket: None,
        status: TerminalStatus::Initialized,
        messages: Vec::new(),
    });

    // Cleanup websocket on tab change
    {
        // let status = status.clone();
        // let ws_state = ws_state.clone();
        // let callback_state = callback_state.clone();
        // let messages = messages.clone();

        // use_effect_with(props.selected_tab.clone(), move |_| {
        //     move || {
        //         if *status != TerminalStatus::Initialized {
        //             if let Some(ws) = &ws_state.websocket {
        //                 ws.close().ok(); // Close the WebSocket
        //             }
        //             callback_state.set(None);
        //             if *status != TerminalStatus::Complete && *status != TerminalStatus::Failed {
        //                 status.set(TerminalStatus::Failed);
        //                 messages.dispatch(MsgAction::AddMessage {
        //                     stream: StreamType::Meta,
        //                     message: "# [Process failed]".to_string(),
        //                 });
        //             }
        //         }
        //     }
        // });
    }

    // Update gutter height dynamically
    {
        let screen_dimensions = screen_dimensions.clone();
        let num_lines = num_lines.clone();
        use_effect_with(screen_dimensions.height as usize, move |_| {
            if screen_dimensions.height < 900.0 {
                num_lines.set(13);
            } else {
                num_lines.set(24);
            }
            || ()
        });
    }

    // Sync the scroll of the gutter to the output:
    let onscroll = {
        let terminal_ref = terminal_ref.clone();
        let gutter_ref = gutter_ref.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_| {
            user_attempted_scroll.set(true);
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
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_: MouseEvent| {
            user_attempted_scroll.set(true);
            scroll_to_line(&terminal_ref, 0);
        })
    };
    let scroll_to_bottom = {
        let terminal_ref = terminal_ref.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_: MouseEvent| {
            user_attempted_scroll.set(false);
            scroll_to_line(&terminal_ref, i32::MAX);
        })
    };

    // Reset reinitializes websocket and terminal
    fn cancel_websocket(ws_state: &WebSocketState) {
        if let Some(ws) = &ws_state.websocket {
            ws.send_with_str("\"Cancel\"").ok();
            //ws.close().ok();
        }
    }
    let cancel_process = {
        let ws_state = ws_state.clone();
        Callback::from(move |_: MouseEvent| {
            cancel_websocket(&ws_state);
        })
    };
    let reset_terminal = {
        let ws_state = ws_state.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_: MouseEvent| {
            cancel_websocket(&ws_state);
            user_attempted_scroll.set(false);
            ws_state.dispatch(WebSocketAction::Reset);
        })
    };

    // "Run command" button callback to set up WebSocket and change status
    let run_command = {
        let ws_state = ws_state.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_: MouseEvent| {
            //debug!("Run command button clicked");
            user_attempted_scroll.set(false);
            // Close any existing WebSocket before starting a new one
            if let Some(ws) = &ws_state.websocket {
                //debug!("Closing existing WebSocket");
                ws.close().ok();
            }
            ws_state.dispatch(WebSocketAction::Reset);

            // Attempt to connect the WebSocket
            //debug!("Attempting to connect WebSocket");
            if let Ok(ws) = WebSocket::new("/api/workstation/command_execute/") {
                //debug!("WebSocket connection established");
                let ws_clone = ws.clone();
                ws_state.dispatch(WebSocketAction::Connect(ws));

                let onmessage_callback = {
                    let ws_state = ws_state.clone();
                    Closure::wrap(Box::new(move |event: MessageEvent| {
                        //debug!("Message received from WebSocket");
                        if let Ok(blob) = event.data().dyn_into::<Blob>() {
                            // Handle Blob message
                            let ws_state = ws_state.clone();

                            // Create the FileReader inside the closure so it's not shared
                            let reader = FileReader::new().unwrap();
                            let reader_clone = reader.clone();
                            let onloadend_callback =
                                Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                                    let result = reader_clone.result().unwrap(); // Get the result from FileReader

                                    if let Ok(text) = result.dyn_into::<js_sys::JsString>() {
                                        //debug!(format!("Raw Blob message as text: {}", text));
                                        handle_message(ws_state.clone(), text.into());
                                    } else {
                                        error!("Failed to convert result to text");
                                    }
                                })
                                    as Box<dyn FnMut(_)>);

                            reader.set_onloadend(Some(onloadend_callback.as_ref().unchecked_ref()));
                            reader.read_as_text(&blob).unwrap();
                            onloadend_callback.forget();
                        } else {
                            error!("Received unsupported WebSocket message type");
                        }
                    }) as Box<dyn FnMut(MessageEvent)>)
                };
                fn handle_message(ws_state: UseReducerHandle<WebSocketState>, msg: String) {
                    //debug!(format!("ServerMsg: {}", msg));
                    match from_str::<ServerMsg>(&msg) {
                        Ok(server_msg) => match server_msg {
                            ServerMsg::Ping => {
                                ws_state.dispatch(WebSocketAction::SendPong);
                            }
                            ServerMsg::PingReport(r) => {
                                ws_state.dispatch(WebSocketAction::ReceivePingReport(r));
                            }
                            ServerMsg::Process(p) => {
                                ws_state.dispatch(WebSocketAction::ReceiveProcess(p.id));
                            }
                            ServerMsg::ProcessOutput(o) => {
                                ws_state.dispatch(WebSocketAction::ReceiveProcessOutput(
                                    o.stream, o.line,
                                ));
                            }
                            ServerMsg::ProcessComplete(c) => {
                                ws_state.dispatch(WebSocketAction::ReceiveProcessComplete(
                                    c.id.to_string(),
                                    c.code.try_into().unwrap_or(128),
                                ));
                            }
                            _ => {}
                        },
                        Err(e) => {
                            error!(format!("Failed to parse message: {}, error: {}", msg, e));
                            ws_state.dispatch(WebSocketAction::Failed(msg));
                        }
                    }
                }
                ws_clone.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
                onmessage_callback.forget();

                let onerror_callback = {
                    let ws_state = ws_state.clone();
                    Closure::wrap(Box::new(move |error: ErrorEvent| {
                        debug!(format!("WebSocket error: {}", error.message()));
                        ws_state
                            .dispatch(WebSocketAction::Failed(format!("{:?}", error.message())));
                    }) as Box<dyn FnMut(ErrorEvent)>)
                };
                ws_clone.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
                onerror_callback.forget();
            } else {
                debug!("Failed to establish WebSocket connection");
            }
        })
    };

    let mut line_number_gutter = 1;
    let mut line_number_output = 1;

    let settings_link = html! {
        <Button>{"📻️ Settings"}</Button>
    };
    let toggle_line_numbers = {
        let show_line_numbers = show_line_numbers.clone();
        Callback::from(move |value: bool| {
            show_line_numbers.set(value);
            LocalStorage::set(SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY, value)
                .expect("Failed to store setting in local storage");
        })
    };
    let toggle_background_change = {
        let background_color_change = background_color_change.clone();
        Callback::from(move |value: bool| {
            background_color_change.set(value);
            LocalStorage::set(BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY, value)
                .expect("Failed to store setting in local storage");
        })
    };
    let update_success_color = Callback::from(move |event: Event| {
        let input: HtmlInputElement = event.target_unchecked_into();
        background_color_success_clone.set(input.value());
        LocalStorage::set(BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY, input.value())
            .expect("Failed to store setting in local storage");
    });

    let update_failure_color = Callback::from(move |event: Event| {
        let input: HtmlInputElement = event.target_unchecked_into();
        background_color_failure_clone.set(input.value());
        LocalStorage::set(BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY, input.value())
            .expect("Failed to store setting in local storage");
    });
    let settings_panel = html_nested!(
        <PopoverBody
            header={html!("")}
            footer={html!("")}
        >
            <List r#type={ListType::Bordered}>
                <ListItem>
                    <Switch
                        label="Show line numbers"
                        checked={*show_line_numbers}
                        onchange={toggle_line_numbers.clone()}
                    />
                </ListItem>
                <ListItem>
                    <Switch
                        label="Background color indicates success/failure"
                        checked={*background_color_change}
                        onchange={toggle_background_change.clone()}
                    />
                </ListItem>
                <ListItem>
                    <div class={if *background_color_change { "visible flex-container" } else { "hidden" }}>
                        <input
                            type="color"
                            id="successColor"
                            name="successColor"
                            value={<std::string::String as Clone>::clone(&*background_color_success)}
                            onchange={update_success_color.clone()}
                        />
                        <label for="successColor">{"Success Color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                    <div class={if *background_color_change { "visible flex-container" } else { "hidden" }}>
                        <input
                            type="color"
                            id="failureColor"
                            name="failureColor"
                            value={<std::string::String as Clone>::clone(&*background_color_failure)}
                            onchange={update_failure_color.clone()}
                        />
                        <label for="failureColor">{"Failure Color"}</label>
                    </div>
                </ListItem>
            </List>
        </PopoverBody>
    );

    {
        let user_attempted_scroll = user_attempted_scroll.clone();
        let terminal_ref = terminal_ref.clone();
        let messages_len = ws_state.messages.len();
        use_effect_with(messages_len, move |_| {
            if !*user_attempted_scroll {
                scroll_to_line(&terminal_ref, i32::MAX);
            }
            || ()
        });
    }

    let output_background_color = match *background_color_change {
        true => match ws_state.status {
            TerminalStatus::Complete => &background_color_success,
            TerminalStatus::Failed => &background_color_failure,
            _ => &background_color_normal,
        },
        false => &background_color_normal,
    };

    html! {
        <div class="terminal">
            <div class="toolbar pf-u-display-flex pf-u-justify-content-space-between">
                <div class="pf-u-display-flex">
                        if ws_state.status == TerminalStatus::Initialized {
                          <Button onclick={run_command.clone()}>{"🚀 Run command"}</Button>
                        } else if ws_state.status == TerminalStatus::Processing {
                          <Button onclick={cancel_process.clone()}>{"🛑 Stop"}</Button>
                        } else if ws_state.status == TerminalStatus::Complete {
                            <Button onclick={reset_terminal.clone()}>{"👍️ Done"}</Button>
                        } else {
                            <Button onclick={reset_terminal.clone()}>{"💥 Reset"}</Button>
                        }
            </div>
                <div class="pf-u-display-flex">
                    <Popover target={settings_link} body={settings_panel} />
                    <Button onclick={scroll_to_top.clone()}>{"⬆️ Top"}</Button>
                    <Button onclick={scroll_to_bottom.clone()}>{"⬇️ Bottom"}</Button>
                </div>
            </div>
            <div class="content">
                if *show_line_numbers && ws_state.status != TerminalStatus::Initialized {
                    <div class="gutter" ref={gutter_ref} style={format!("max-height: {}em", *num_lines)}>
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
        <div class="output" ref={terminal_ref} {onscroll} style={format!("max-height: {}em; background-color: {}", *num_lines, **output_background_color)}>
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
