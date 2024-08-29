use crate::components::markdown::{MarkdownContent};
use crate::{app::WindowDimensions, pages::workstation::WorkstationTab};
use dry_console_dto::script::ScriptEntry;
use dry_console_dto::websocket::Command;
use dry_console_dto::websocket::PingReport;
use dry_console_dto::websocket::ServerMsg;
use dry_console_dto::websocket::StreamType;
use gloo::console::debug;
use gloo::console::error;
use gloo::net::http::Request;
use gloo_storage::LocalStorage;
use gloo_storage::Storage;
use patternfly_yew::prelude::*;
use serde_json::from_str;
use std::rc::Rc;
use ulid::Ulid;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys;
use web_sys::js_sys::JsString;
use web_sys::js_sys::Promise;
use web_sys::js_sys::Reflect;
use web_sys::window;
use web_sys::Blob;
use web_sys::FileReader;
use web_sys::HtmlInputElement;
use web_sys::MessageEvent;
use web_sys::{HtmlElement, WebSocket};
use yew::prelude::*;

const SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY: &str = "terminal:show_line_numbers";
const BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY: &str = "terminal:background_color_change";
const BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY: &str = "terminal:background_color_success";
const BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY: &str = "terminal:background_color_failure";
const BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY: &str = "terminal:background_color_normal";
const TEXT_COLOR_STDOUT_LOCALSTORAGE_KEY: &str = "terminal:text_color_stdout";
const TEXT_COLOR_STDERR_LOCALSTORAGE_KEY: &str = "terminal:text_color_stderr";

pub fn scroll_to_line(node_ref: &NodeRef, line_number: i32) {
    if let Some(element) = node_ref.cast::<web_sys::HtmlElement>() {
        //debug!(element.clone());
        // Calculate the scroll position based on line height and line number
        let line_height = 20; // Adjust this according to your CSS
        let scroll_position = if line_number <= 0 {
            0
        } else if line_number == i32::MAX {
            element.scroll_height()
        } else {
            line_number * line_height
        };
        //debug!(format!("scroll! {}", scroll_position));
        element.set_scroll_top(scroll_position);
    }
}

#[derive(Debug, PartialEq)]
struct WebSocketState {
    websocket: Option<WebSocket>,
    script_entry: Option<ScriptEntry>,
    status: TerminalStatus,
    messages: Vec<(StreamType, String)>,
    error: String,
}
// Reducer actions to manage WebSocketState
#[derive(Debug)]
enum WebSocketAction {
    Initialize(ScriptEntry),
    Connect(WebSocket),
    ReceivePingReport(PingReport),
    ReceiveProcessOutput(StreamType, String),
    ReceiveProcessComplete(String, usize),
    ReceiveProcess(Ulid),
    Failed(String),
    CriticalError(String),
    Reset,
    SendPong,
}
impl Reducible for WebSocketState {
    type Action = WebSocketAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        //debug!(format!("Reducer called with action: {:?}", action));
        //debug!(format!("Current state before action: {:?}", *self));

        let new_state: Rc<WebSocketState> = match action {
            WebSocketAction::Initialize(script_entry) => {
                //debug!("Action: Initialize");
                WebSocketState {
                    script_entry: Some(script_entry),
                    websocket: None,
                    status: TerminalStatus::Initialized,
                    messages: self.messages.clone(),
                    error: self.error.clone(),
                }
                .into()
            }
            WebSocketAction::Connect(ws) => {
                //debug!("Action: Connect");
                WebSocketState {
                    script_entry: self.script_entry.clone(),
                    websocket: Some(ws),
                    status: TerminalStatus::Connecting,
                    messages: self.messages.clone(),
                    error: self.error.clone(),
                }
                .into()
            }
            WebSocketAction::ReceivePingReport(_r) => {
                //debug!(format!("Action: ReceivePingReport {:?}", r));
                WebSocketState {
                    script_entry: self.script_entry.clone(),
                    websocket: self.websocket.clone(),
                    status: if self.status == TerminalStatus::Connecting {
                        if let Some(ws) = &self.websocket {
                            if let Ok(serialized_msg) = serde_json::to_string(&Command {
                                id: self.script_entry.clone().unwrap().id,
                            }) {
                                //debug!(format!("sending command serialized: {}", serialized_msg));
                                ws.send_with_str(&serialized_msg).ok();
                            }
                        }
                        TerminalStatus::Ready
                    } else {
                        self.status.clone()
                    },
                    messages: self.messages.clone(),
                    error: self.error.clone(),
                }
                .into()
            }
            WebSocketAction::ReceiveProcess(_id) => {
                //debug!(format!("Action: ReceiveProcess, id: {:?}", id));
                WebSocketState {
                    script_entry: self.script_entry.clone(),
                    websocket: self.websocket.clone(),
                    status: TerminalStatus::Processing,
                    messages: self.messages.clone(),
                    error: self.error.clone(),
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
                    script_entry: self.script_entry.clone(),
                    websocket: self.websocket.clone(),
                    status: self.status.clone(),
                    messages,
                    error: self.error.clone(),
                }
                .into()
            }
            WebSocketAction::ReceiveProcessComplete(_id, code) => {
                //debug!(format!(
                //                    "Action: ReceiveProcessComplete, id: {:?}, code: {}",
                //    id, code
                //));
                WebSocketState {
                    script_entry: self.script_entry.clone(),
                    websocket: self.websocket.clone(),
                    status: if code == 0 {
                        TerminalStatus::Complete
                    } else {
                        TerminalStatus::Failed
                    },
                    messages: self.messages.clone(),
                    error: self.error.clone(),
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
                    script_entry: self.script_entry.clone(),
                    websocket: None,
                    status: TerminalStatus::Failed,
                    messages,
                    error: self.error.clone(),
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
                    script_entry: self.script_entry.clone(),
                    websocket: None,
                    status: TerminalStatus::Initialized,
                    messages: Vec::new(),
                    error: self.error.clone(),
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
            WebSocketAction::CriticalError(e) => {
                //debug!("Action: CriticalError");
                WebSocketState {
                    script_entry: None,
                    websocket: None,
                    status: TerminalStatus::Critical,
                    messages: Vec::new(),
                    error: e,
                }
                .into()
            }
        };

        //debug!(format!("New state after action: {:?}", *new_state));
        new_state
    }
}

#[derive(PartialEq, Debug, Clone)]
enum TerminalStatus {
    Uninitialized,
    Initialized,
    Connecting,
    Ready,
    Processing,
    Failed,
    Critical,
    Complete,
}

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub script: String,
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
    pub on_done: Option<Callback<MouseEvent>>,
}
impl TerminalOutputProps {
    pub fn default_on_done() -> Callback<MouseEvent> {
        Callback::from(|_| {})
    }
}

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let screen_dimensions = use_context::<WindowDimensions>().expect("no ctx found");
    let num_lines = use_state(|| 1);
    let show_line_numbers = use_state(|| {
        LocalStorage::get::<bool>(SHOW_LINE_NUMBERS_LOCALSTORAGE_KEY).unwrap_or(false)
    });
    let background_color_change = use_state(|| {
        LocalStorage::get::<bool>(BACKGROUND_COLOR_CHANGE_LOCALSTORAGE_KEY).unwrap_or(true)
    });
    let background_color_success = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_SUCCESS_LOCALSTORAGE_KEY)
            .unwrap_or("#275346".to_string())
    });
    let background_color_success_clone = background_color_success.clone();
    let background_color_failure = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_FAILURE_LOCALSTORAGE_KEY)
            .unwrap_or("#712121".to_string())
    });
    let background_color_failure_clone = background_color_failure.clone();
    let background_color_normal = use_state(|| {
        LocalStorage::get::<String>(BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY)
            .unwrap_or("#000000".to_string())
    });
    let background_color_normal_clone = background_color_normal.clone();
    let text_color_stdout = use_state(|| {
        LocalStorage::get::<String>(TEXT_COLOR_STDOUT_LOCALSTORAGE_KEY)
            .unwrap_or("#ffffff".to_string())
    });
    let text_color_stdout_clone = text_color_stdout.clone();
    let text_color_stderr = use_state(|| {
        LocalStorage::get::<String>(TEXT_COLOR_STDERR_LOCALSTORAGE_KEY)
            .unwrap_or("#dc8add".to_string())
    });
    let text_color_stderr_clone = text_color_stderr.clone();
    let user_attempted_scroll = use_state(|| false);
    let terminal_ref = use_node_ref();
    let terminal_content_ref = use_node_ref();
    let gutter_ref = use_node_ref();

    let ws_state = use_reducer(|| WebSocketState {
        script_entry: None,
        websocket: None,
        status: TerminalStatus::Uninitialized,
        messages: Vec::new(),
        error: "".to_string(),
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
        let terminal_content_ref = terminal_content_ref.clone();
        let gutter_ref = gutter_ref.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_| {
            user_attempted_scroll.set(true);
            if let (Some(terminal), Some(gutter)) = (
                terminal_content_ref.cast::<HtmlElement>(),
                gutter_ref.cast::<HtmlElement>(),
            ) {
                let scroll_top = terminal.scroll_top();
                gutter.set_scroll_top(scroll_top);
            }
        })
    };
    let scroll_to_top = {
        let content_ref = terminal_content_ref.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_: MouseEvent| {
            user_attempted_scroll.set(true);
            scroll_to_line(&content_ref, 0);
        })
    };
    let scroll_to_bottom = {
        let content_ref = terminal_content_ref.clone();
        let user_attempted_scroll = user_attempted_scroll.clone();
        Callback::from(move |_: MouseEvent| {
            user_attempted_scroll.set(false);
            scroll_to_line(&content_ref, i32::MAX);
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

            let ws = WebSocket::new("/api/workstation/command_execute/").unwrap();
            let ws_clone = ws.clone();
            ws_state.dispatch(WebSocketAction::Connect(ws));

            // Set up the onerror callback to catch connection errors
            let onerror_callback = {
                let ws_state = ws_state.clone();
                Closure::wrap(Box::new(move |error: ErrorEvent| {
                    debug!(format!("WebSocket error: {}", error.message()));
                    ws_state.dispatch(WebSocketAction::Failed(format!("{:?}", error.message())));
                }) as Box<dyn FnMut(ErrorEvent)>)
            };
            ws_clone.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
            onerror_callback.forget();

            // Set up the onmessage callback
            let onmessage_callback = {
                let ws_state = ws_state.clone();
                Closure::wrap(Box::new(move |event: MessageEvent| {
                    if let Ok(blob) = event.data().dyn_into::<Blob>() {
                        let ws_state = ws_state.clone();

                        let reader = FileReader::new().unwrap();
                        let reader_clone = reader.clone();
                        let onloadend_callback =
                            Closure::wrap(Box::new(move |_: web_sys::ProgressEvent| {
                                let result = reader_clone.result().unwrap();

                                if let Ok(text) = result.dyn_into::<js_sys::JsString>() {
                                    handle_message(ws_state.clone(), text.into());
                                } else {
                                    error!("Failed to convert result to text");
                                }
                            }) as Box<dyn FnMut(_)>);

                        reader.set_onloadend(Some(onloadend_callback.as_ref().unchecked_ref()));
                        reader.read_as_text(&blob).unwrap();
                        onloadend_callback.forget();
                    } else {
                        error!("Received unsupported WebSocket message type");
                    }
                }) as Box<dyn FnMut(MessageEvent)>)
            };

            fn handle_message(ws_state: UseReducerHandle<WebSocketState>, msg: String) {
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
                            ws_state
                                .dispatch(WebSocketAction::ReceiveProcessOutput(o.stream, o.line));
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
    let update_normal_background_color = Callback::from(move |event: Event| {
        let input: HtmlInputElement = event.target_unchecked_into();
        background_color_normal_clone.set(input.value());
        LocalStorage::set(BACKGROUND_COLOR_NORMAL_LOCALSTORAGE_KEY, input.value())
            .expect("Failed to store setting in local storage");
    });
    let update_stdout_text_color = Callback::from(move |event: Event| {
        let input: HtmlInputElement = event.target_unchecked_into();
        text_color_stdout_clone.set(input.value());
        LocalStorage::set(TEXT_COLOR_STDOUT_LOCALSTORAGE_KEY, input.value())
            .expect("Failed to store setting in local storage");
    });
    let update_stderr_text_color = Callback::from(move |event: Event| {
        let input: HtmlInputElement = event.target_unchecked_into();
        text_color_stderr_clone.set(input.value());
        LocalStorage::set(TEXT_COLOR_STDERR_LOCALSTORAGE_KEY, input.value())
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
                    <div class={"visible flex-container"}>
                        <input
                            type="color"
                            id="normalBackgroundColor"
                            name="normalBackgroundColor"
                            value={<std::string::String as Clone>::clone(&*background_color_normal)}
                            onchange={update_normal_background_color.clone()}
                        />
                        <label for="normalBackgroundColor">{"Terminal background color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                    <div class={"visible flex-container"}>
                        <input
                            type="color"
                            id="normalTextColor"
                            name="normalTextColor"
                            value={<std::string::String as Clone>::clone(&*text_color_stdout)}
                            onchange={update_stdout_text_color.clone()}
                        />
                        <label for="normalTextColor">{"Terminal stdout color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                    <div class={"visible flex-container"}>
                        <input
                            type="color"
                            id="normalTextColor"
                            name="normalTextColor"
                            value={<std::string::String as Clone>::clone(&*text_color_stderr)}
                            onchange={update_stderr_text_color.clone()}
                        />
                        <label for="normalTextColor">{"Terminal stderr color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                    <Switch
                        label="Background color changes on success / failure"
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
                        <label for="successColor">{"Color to indicate success"}</label>
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
                        <label for="failureColor">{"Color to indicate failure"}</label>
                    </div>
                </ListItem>
            </List>
        </PopoverBody>
    );
    fn copy_code(
        code_block_ref: NodeRef,
        set_button_text: yew::UseStateHandle<String>,
    ) -> Callback<MouseEvent> {
        Callback::from(move |_| {
            if let Some(element) = code_block_ref.cast::<HtmlElement>() {
                //debug!(format!("element: {:?}", element));
                if let Some(content_element) = element.query_selector(".content").unwrap() {
                    // Cast `Element` to `HtmlElement` to use `inner_text()`
                    if let Ok(content) = content_element.dyn_into::<HtmlElement>() {
                        let text = content.inner_text();
                        if let Some(window) = window() {
                            let navigator = window.navigator();
                            let clipboard =
                                Reflect::get(&navigator, &JsString::from("clipboard")).unwrap();

                            if clipboard.is_undefined() {
                                error!("Clipboard API is not supported in this browser");
                            } else {
                                let clipboard: web_sys::Clipboard = clipboard.dyn_into().unwrap();
                                let promise: Promise = clipboard.write_text(&text);

                                let set_button_text = set_button_text.clone();
                                let future = JsFuture::from(promise);
                                wasm_bindgen_futures::spawn_local(async move {
                                    if future.await.is_ok() {
                                        set_button_text.set("✅".to_string());
                                        wasm_bindgen_futures::spawn_local(async move {
                                            gloo::timers::future::TimeoutFuture::new(2000).await;
                                            set_button_text.set("📋".to_string());
                                        });
                                    } else {
                                        error!("Failed to copy text");
                                    }
                                });
                            }
                        } else {
                            error!("window not found.");
                        }
                    } else {
                        error!("Failed to cast content_element to HtmlElement.");
                    }
                } else {
                    error!("Failed to find .content.");
                }
            } else {
                error!("code block not found.");
            }
        })
    }

    #[derive(Properties, PartialEq, Clone)]
    pub struct CommandAreaProps {
        pub script: String,
        pub description: String,
        pub background_color: String,
        pub foreground_color: String,
    }
    #[function_component(CommandArea)]
    fn command_area(props: &CommandAreaProps) -> Html {
        let CommandAreaProps {
            script,
            description,
            background_color,
            foreground_color,
        } = props;
        let code_block_ref = NodeRef::default();
        let button_text = use_state(|| "📋".to_string());
        let expanded = use_state_eq(|| false);
        let ontoggle = use_callback(expanded.clone(), |(), expanded| {
            expanded.set(!**expanded);
        });
        html! {
            <div class="command_area" style="position: relative;">
                <div class="header">
                {"💻️ Run bash script"}
                </div>
                <Stack gutter=true>
                <StackItem>
                <MarkdownContent source={description.to_string()}/>
                <ExpandableSectionToggle toggle_text_expanded={"Hide script"} toggle_text_hidden={"Show script"} {ontoggle} expanded={*expanded} direction={ExpandableSectionToggleDirection::Down}/>
                </StackItem>
                <StackItem>
                <ExpandableSection detached=true expanded={*expanded}>
                <div class="code_container" ref={code_block_ref.clone()}>
                <div class="content" style={format!("background-color: {}; color: {}", background_color, foreground_color)}>
                <CodeBlock>
                <CodeBlockCode>{script}</CodeBlockCode>
                </CodeBlock>
            </div>
            <button title="Copy script" class="copy-button" onclick={copy_code(code_block_ref.clone(), button_text.clone())}><div class="copy-button-text">{ (*button_text).clone() }</div></button>
                </div>
                </ExpandableSection>
                </StackItem>
                </Stack>
            </div>
        }
    }

    {
        let user_attempted_scroll = user_attempted_scroll.clone();
        let content_ref = terminal_content_ref.clone();
        let messages_len = ws_state.messages.len();
        use_effect_with(messages_len, move |_| {
            if !*user_attempted_scroll {
                scroll_to_line(&content_ref, i32::MAX);
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

    let output_stdout_color = &text_color_stdout;
    let output_copy_button_text = use_state(|| "📋".to_string());

    // Initialize script entry
    {
        let ws_state = ws_state.clone();
        let script = props.script.clone();
        use_effect_with(ws_state.status.clone(), move |status| {
            if *status == TerminalStatus::Uninitialized {
                let ws_state = ws_state.clone();
                spawn_local(async move {
                    let response = Request::get(&format!("/api/workstation/command/{}/", script))
                        .send()
                        .await;

                    match response {
                        Ok(resp) => {
                            if let Ok(data) = resp.json::<ScriptEntry>().await {
                                ws_state.dispatch(WebSocketAction::Initialize(data));
                            } else {
                                match resp.status() {
                                    404 => ws_state.dispatch(WebSocketAction::CriticalError(
                                        "Could not find script entry.".to_string(),
                                    )),
                                    _ => ws_state.dispatch(WebSocketAction::CriticalError(
                                        "Failed to fetch script entry.".to_string(),
                                    )),
                                }
                                // ;
                            }
                        }
                        Err(e) => {
                            ws_state
                                .dispatch(WebSocketAction::Failed(format!("Fetch error: {:?}", e)));
                        }
                    }
                });
            }

            || ()
        });
    }
    let script_entry = ws_state
        .script_entry
        .clone()
        .unwrap_or(ScriptEntry::default());

    let done = {
        let reset_terminal = reset_terminal.clone();
        let on_done = props.on_done.clone();
        Callback::from(move |e: MouseEvent| {
            reset_terminal.emit(e.clone());
            if let Some(on_done) = on_done.clone() {
                on_done.emit(e.clone());
            }
        })
    };

    html! {
        <div class="terminal">
        if ws_state.status == TerminalStatus::Critical {
            <Alert title="Error" r#type={AlertType::Danger}>{ws_state.error.clone()}</Alert>
        } else if ws_state.status == TerminalStatus::Uninitialized {
            <LoadingState/>
        } else {
            <CommandArea description={script_entry.description.clone()} script={script_entry.script} background_color={(*background_color_normal).clone()} foreground_color={(*text_color_stdout).clone()}/>
            <div class="toolbar pf-u-display-flex pf-u-justify-content-space-between">
            <div class="pf-u-display-flex">
                        if ws_state.status == TerminalStatus::Initialized {
                          <Button onclick={run_command.clone()}>{"🚀 Run script"}</Button>
                        } else if ws_state.status == TerminalStatus::Processing {
                          <Button onclick={cancel_process.clone()}>{"🛑 Stop"}</Button>
                        } else if ws_state.status == TerminalStatus::Complete {
                            <Button onclick={done.clone()}>{"👍️ Done"}</Button>
                        } else if ws_state.status == TerminalStatus::Connecting {
                            <Button onclick={reset_terminal.clone()}>{"⏳️ Reset"}</Button>
                        } else {
                            <Button onclick={reset_terminal.clone()}>{"💥 Reset"}</Button>
                        }
            </div>
                <div class="pf-u-display-flex">
                    <Popover target={settings_link} body={settings_panel} />
                    if ws_state.status != TerminalStatus::Initialized {
                      <Button onclick={scroll_to_top.clone()}>{"⬆️ Top"}</Button>
                      <Button onclick={scroll_to_bottom.clone()}>{"⬇️ Bottom"}</Button>
                    }
                </div>
            </div>
            <div class="terminal_display" ref={terminal_ref.clone()}>
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
        if ws_state.status == TerminalStatus::Complete || ws_state.status == TerminalStatus::Failed {
            <button title="Copy output" class="copy-button" onclick={copy_code(terminal_ref.clone(), output_copy_button_text.clone())}><div class="copy-button-text">{ (*output_copy_button_text).clone() }</div></button>
        }
        <div class="content" ref={terminal_content_ref.clone()} {onscroll} style={format!("max-height: {}em; background-color: {}; color: {}", *num_lines, **output_background_color, **output_stdout_color)}>
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
                let style = match stream {
                    StreamType::Stderr => format!("color: {}", *text_color_stderr),
                    StreamType::Stdout => "".to_string(),
                    StreamType::Meta => "".to_string()
                };
                html!{
                    <span id={id} class={class_name} {style}>{message}</span>
                }
            })
        }
        </div>
        </div>
        }
        </div>
    }
}

#[function_component(LoadingState)]
fn loading_state() -> Html {
    html! {
        <Card>
            <CardTitle><p><h1>{"⌛️ Loading ..."}</h1></p></CardTitle>
            <CardBody>
                <div class="flex-center">
                    <Spinner size={SpinnerSize::Custom(String::from("80px"))} aria_label="Contents of the custom size example" />
                </div>
            </CardBody>
        </Card>
    }
}
