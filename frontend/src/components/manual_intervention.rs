use crate::components::markdown::{markdown_to_html, MarkdownContent};
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

#[derive(Properties, PartialEq, Clone)]
pub struct ManualInterventionProps {
    pub description: String,
    pub script: String,
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}
#[function_component(ManualIntervention)]
pub fn manual_intervention(props: &ManualInterventionProps) -> Html {
    let terminal_ref = use_node_ref();
    let terminal_content_ref = use_node_ref();

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
    let code_block_ref = NodeRef::default();
    let button_text = use_state(|| "📋".to_string());

    html! {
        <div class="manual_intervention">
                        <div class="command_area" style="position: relative;">
                <div class="header">
                {"👷 Manual intervention required"}
                </div>
                <Stack gutter=true>
            <StackItem>
            <p>{props.description.clone()}</p>
            <br/>
            <ul>
            <li>{"Open your workstation's terminal application."}</li>
            <li>{"Click the clipboard button to copy the script below."}</li>
            <li>{"Paste it into your terminal and press Enter to run it."}</li>
            </ul>
            </StackItem>
                <StackItem>
                <div class="code_container" ref={code_block_ref.clone()}>
                <div class="content">
                <CodeBlock>
                <CodeBlockCode>
                    {props.script.clone()}
                </CodeBlockCode>
                </CodeBlock>
            </div>
            <button title="Copy script" class="copy-button" onclick={copy_code(code_block_ref.clone(), button_text.clone())}><div class="copy-button-text">{ (*button_text).clone() }</div></button>
                </div>
                </StackItem>
                </Stack>
            </div>
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
