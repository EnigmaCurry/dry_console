use crate::components::color_picker::ColorPicker;
use crate::components::loading_state::LoadingState;
use crate::components::markdown::MarkdownContent;
use crate::components::terminal_context_provider::TerminalStyleContext;
use crate::components::terminal_context_provider::*;
use crate::{app::WindowDimensions, pages::workstation::WorkstationTab};
use dry_console_dto::script::ScriptEntry;
use dry_console_dto::websocket::Command;
use dry_console_dto::websocket::PingReport;
use dry_console_dto::websocket::ServerMsg;
use dry_console_dto::websocket::StreamType;
use gloo::console::debug;
use gloo::console::error;
use gloo::net::http::Request;
use patternfly_yew::prelude::*;
use serde_json::from_str;
use std::collections::HashMap;
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
pub struct EnvVarListProps {
    pub env_vars: Vec<EnvVarProps>,
}

#[function_component(EnvVarList)]
pub fn env_var_list(props: &EnvVarListProps) -> Html {
    html! {
        <div class="env_var_list">
            <h3>{"Configure script environment:"}</h3>
            { for props.env_vars.iter().map(|env_var| html! {
                <EnvVar name={env_var.name.clone()} description={env_var.description.clone()} on_value_change={env_var.on_value_change.clone()} is_valid={env_var.is_valid} help={env_var.help.clone()}/>
            }) }
        </div>
    }
}

#[derive(Debug, Properties, PartialEq, Clone, Default)]
pub struct EnvVarProps {
    pub name: String,
    pub description: String,
    pub is_valid: bool,
    pub help: Vec<String>,
    #[prop_or_default]
    pub on_value_change: Option<Callback<(String, String)>>,
    #[prop_or_default]
    pub default_value: String,
}

#[function_component(EnvVar)]
pub fn env_var(props: &EnvVarProps) -> Html {
    let env_var_value = use_state(|| "".to_string());
    let is_input_focused = use_state(|| false);
    let is_tooltip_visible = use_state(|| false);
    let name = props.name.clone();
    let description = props.description.clone();
    let on_value_change = props.on_value_change.clone();

    // Create a NodeRef to reference the TextInput element
    let input_ref = use_node_ref();

    // Callback to handle the value change from InputEvent
    let onchange = {
        let env_var_value = env_var_value.clone();
        let name = name.clone();
        let on_value_change = on_value_change.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                let value = input.value();
                env_var_value.set(value.clone());
                if let Some(on_value_change) = &on_value_change {
                    on_value_change.emit((name.clone(), value));
                }
            } else {
                debug!("Failed to cast InputEvent target to HtmlInputElement");
            }
        })
    };

    // Use effect to attach focus and blur events using use_effect_with
    let is_input_focused_clone_for_focus = is_input_focused.clone();
    let is_input_focused_clone_for_blur = is_input_focused.clone();
    let is_tooltip_visible_clone_for_focus = is_tooltip_visible.clone();
    let is_tooltip_visible_clone_for_blur = is_tooltip_visible.clone();
    use_effect_with(input_ref.clone(), move |input_ref| {
        let input_element = input_ref.cast::<HtmlInputElement>();
        if let Some(input_element) = input_element {
            let focus_closure = Closure::wrap(Box::new(move || {
                is_input_focused_clone_for_focus.set(true);
                is_tooltip_visible_clone_for_focus.set(true);
            }) as Box<dyn Fn()>);

            let blur_closure = Closure::wrap(Box::new(move || {
                is_input_focused_clone_for_blur.set(false);
                is_tooltip_visible_clone_for_blur.set(false);
            }) as Box<dyn Fn()>);

            input_element.set_onfocus(Some(focus_closure.as_ref().unchecked_ref()));
            input_element.set_onblur(Some(blur_closure.as_ref().unchecked_ref()));

            // Call `forget` to ensure the closures are not dropped prematurely
            focus_closure.forget();
            blur_closure.forget();

            // Log when the cleanup function is set
            Box::new(move || {
                input_element.set_onfocus(None);
                input_element.set_onblur(None);
            }) as Box<dyn Fn()>
        } else {
            Box::new(|| ()) as Box<dyn Fn()>
        }
    });

    // Callback to handle Button click and focus the TextInput
    let on_focus_input = {
        let input_ref = input_ref.clone();
        Callback::from(move |_| {
            if let Some(input_element) = input_ref.cast::<HtmlInputElement>() {
                input_element.focus().unwrap();
            }
        })
    };

    let validation_text = match (props.is_valid, env_var_value.is_empty()) {
        (true, _) => "✅",
        (false, true) => "✍️",
        (false, false) => "⁉️",
    };

    let validation_help = match (props.is_valid, env_var_value.is_empty()) {
        (true, _) => format!("{name} looks good! ✅"),
        (false, true) => format!("Please enter a value for {name}. ✍️"),
        (false, false) => format!("{name} is invalid ⁉️"),
    };

    let show_tooltip = *is_tooltip_visible;

    let tooltip_classes = match props.is_valid {
        true => "pf-v5-c-tooltip pf-m-bottom-left tooltip valid",
        false => "pf-v5-c-tooltip pf-m-bottom-left tooltip",
    };

    // Render the tooltip with validation help and additional help messages
    html! {
        <div class="env_var_entry">
            <Form>
                <FormGroup label={format!("{name} - {description}")} required=true>
                    <div class="validated_input">
                        <div class="validation" style="position: relative;">
                            <div>
                                <Button tabindex={Some(-1)} onclick={on_focus_input}>
                                    {validation_text}
                                </Button>
                            </div>
                            { if show_tooltip {
                                html! {
                                    <div class={tooltip_classes} role="tooltip">
                                        <div class="pf-v5-c-tooltip__arrow"></div>
                                        <div class="pf-v5-c-tooltip__content">
                                            {validation_help}
                                            <ul>
                                                { for props.help.iter().map(|help_text| html! {
                                                    <li>{help_text}</li>
                                                }) }
                                            </ul>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                        <TextInput
                            required=true
                            value={(*env_var_value).clone()}
                            oninput={onchange}
                            r#ref={input_ref}
                        />
                    </div>
                </FormGroup>
            </Form>
        </div>
    }
}

pub trait IsEnvVar {}

impl IsEnvVar for EnvVar {}

#[derive(Properties, PartialEq)]
pub struct TerminalOutputProps {
    pub script: String,
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
    pub on_done: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub children: Children,
}

impl TerminalOutputProps {
    pub fn default_on_done() -> Callback<MouseEvent> {
        Callback::from(|_| {})
    }
}

#[function_component(TerminalOutput)]
pub fn terminal_output(props: &TerminalOutputProps) -> Html {
    let screen_dimensions = use_context::<WindowDimensions>().expect("no ctx found");
    let env_vars = use_state(HashMap::new);
    let style_ctx = use_context::<TerminalStyleContext>().expect("No TerminalStyleContext found");
    let style = style_ctx.get_settings();
    let num_lines = use_state(|| 1);
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

    let default_on_env_var_change = {
        let env_vars = env_vars.clone();
        Callback::from(move |(name, value): (String, String)| {
            let mut vars = (*env_vars).clone();
            vars.insert(name, value);
            env_vars.set(vars);
        })
    };

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
    let toggle_line_numbers_cb = toggle_line_numbers(style.show_line_numbers.clone());
    let toggle_meta_stream_cb = toggle_meta_stream(style.show_meta_stream.clone());
    let toggle_background_change_cb =
        toggle_background_change(style.background_color_change.clone());
    let update_success_color_cb = update_success_color(style.background_color_success.clone());
    let update_failure_color_cb = update_failure_color(style.background_color_failure.clone());
    let update_normal_background_color_cb =
        update_normal_background_color(style.background_color_normal.clone());
    let update_stdout_text_color_cb = update_stdout_text_color(style.text_color_stdout.clone());
    let update_stderr_text_color_cb = update_stderr_text_color(style.text_color_stderr.clone());

    let settings_panel = html_nested!(
        <PopoverBody
            header={html!("")}
            footer={html!("")}
        >
            <List r#type={ListType::Bordered}>
                <ListItem>
                    <Switch
                        label="Show line numbers"
                        checked={*style.show_line_numbers.clone()}
                        onchange={toggle_line_numbers_cb.clone()}
                    />
                </ListItem>
                <ListItem>
                    <Switch
                        label="Show meta stream"
                        checked={*style.show_meta_stream.clone()}
                        onchange={toggle_meta_stream_cb.clone()}
                    />
                </ListItem>
                <ListItem>
                   <div class={"visible flex-container"}>
                   <ColorPicker
                      onchange={update_normal_background_color_cb.clone()}
                      color={(*style.background_color_normal).clone()}
                     />
                          <label for="normalBackgroundColor">{"Terminal background color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                   <div class={"visible flex-container"}>
                   <ColorPicker
                      onchange={update_stdout_text_color_cb.clone()}
                      color={(*style.text_color_stdout).clone()}
                     />
                          <label for="stdoutTextColor">{"Terminal stdout color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                   <div class={"visible flex-container"}>
                   <ColorPicker
                      onchange={update_stderr_text_color_cb.clone()}
                      color={(*style.text_color_stderr).clone()}
                     />
                          <label for="stderrTextColor">{"Terminal stderr color"}</label>
                    </div>
                </ListItem>
                <ListItem>
                    <Switch
                        label="Background color changes on success / failure"
                        checked={*style.background_color_change}
                        onchange={toggle_background_change_cb.clone()}
                    />
                </ListItem>
                <ListItem>
                    <div class={if *style.background_color_change { "visible flex-container" } else { "hidden" }}>
                   <ColorPicker
                      onchange={update_success_color_cb.clone()}
                      color={(*style.background_color_success).clone()}
                     />
                          <label for="stderrTextColor">{"Color to indicate success"}</label>
                    </div>
                </ListItem>
                <ListItem>
                    <div class={if *style.background_color_change { "visible flex-container" } else { "hidden" }}>
                   <ColorPicker
                      onchange={update_failure_color_cb.clone()}
                      color={(*style.background_color_failure).clone()}
                     />
                          <label for="stderrTextColor">{"Color to indicate failure"}</label>
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

    let output_background_color = match *style.background_color_change {
        true => match ws_state.status {
            TerminalStatus::Complete => &style.background_color_success,
            TerminalStatus::Failed => &style.background_color_failure,
            _ => &style.background_color_normal,
        },
        false => &style.background_color_normal,
    };

    let output_stdout_color = &style.text_color_stdout;
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

    let code_block_ref = NodeRef::default();
    let copy_code_button_text = use_state(|| "📋".to_string());
    let code_block_expanded = use_state_eq(|| false);
    let code_block_ontoggle = use_callback(code_block_expanded.clone(), |(), expanded| {
        expanded.set(!**expanded);
    });

    let terminal_classes = match ws_state.status {
        TerminalStatus::Initialized => "terminal_display hidden",
        _ => "terminal_display",
    };

    html! {
        <div class="terminal">
            if ws_state.status == TerminalStatus::Critical {
                <Alert title="Error" r#type={AlertType::Danger}>{ws_state.error.clone()}</Alert>
            } else if ws_state.status == TerminalStatus::Uninitialized {
                <LoadingState/>
            } else {
                <div class="command_area" style="position: relative;">
                    <div class="header">
                {"💻️ Run bash script"}
                </div>
                    <Stack gutter=true>
                    <StackItem>
                    <MarkdownContent source={script_entry.description.clone()}/>
                    </StackItem>
                    <StackItem>
                    </StackItem>
                    </Stack>
                    </div>
                    if !props.children.is_empty() {
                        { for props.children.iter() }
                    }
                <ExpandableSectionToggle toggle_text_expanded={"Hide script"} toggle_text_hidden={"Show script"} ontoggle={code_block_ontoggle} expanded={*code_block_expanded} direction={ExpandableSectionToggleDirection::Down}/>
                <ExpandableSection detached=true expanded={*code_block_expanded}>
                <div class="code_container" ref={code_block_ref.clone()}>
                <div class="content" style={format!("background-color: {}; color: {}", (*style.background_color_normal).clone(), (*style.text_color_stdout).clone())}>
                <CodeBlock>
                <CodeBlockCode>{script_entry.script}</CodeBlockCode>
                </CodeBlock>
            </div>
            <button title="Copy script" class="copy-button" onclick={copy_code(code_block_ref.clone(), copy_code_button_text.clone())}><div class="copy-button-text">{ (*copy_code_button_text).clone() }</div></button>
                </div>
                </ExpandableSection>
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
                <div class={terminal_classes} ref={terminal_ref.clone()}>
                    if *style.show_line_numbers && ws_state.status != TerminalStatus::Initialized {
                        <div class="gutter" ref={gutter_ref} style={format!("max-height: {}em", *num_lines)}>
                        {
                            for ws_state.messages.iter().filter_map(|(stream, _message)| {
                                if *stream == StreamType::Meta && !*style.show_meta_stream {
                                    None
                                } else {
                                    let gutter_content = match stream {
                                        StreamType::Stdout => {
                                            let content = line_number_gutter.to_string();
                                            line_number_gutter += 1;
                                            content
                                        }
                                        StreamType::Stderr => "E".to_string(),         // "E" for StdErr
                                        StreamType::Meta => "#".to_string(),           // "#" for Meta (assuming this is the correct symbol)
                                    };
                                    Some(html!{
                                        <div class="gutter-line">{gutter_content}</div>
                                    })
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
                        for ws_state.messages.iter().filter_map(|(stream, message)| {
                            if *stream == StreamType::Meta && !*style.show_meta_stream {
                                None
                            } else {
                                let (class_name, id, style) = match stream {
                                    StreamType::Stdout => {
                                        let id = format!("line-{}", line_number_output);
                                        line_number_output += 1;
                                        ("stream-stdout", id, "".to_string())
                                    },
                                    StreamType::Stderr => ("stream-stderr", "".to_string(), format!("color: {}", *style.text_color_stderr)),
                                    StreamType::Meta => ("stream-meta", "".to_string(), "".to_string()),
                                };
                                Some(html!{
                                    <span id={id} class={class_name} style={style}>{message}</span>
                                })
                            }
                        })
                    }
                    </div>
                </div>
            }
        </div>
    }
}
