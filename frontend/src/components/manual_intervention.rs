use crate::components::loading_state::LoadingState;
use crate::pages::workstation::WorkstationTab;
use dry_console_dto::script::ScriptEntry;
use gloo::console::error;
use patternfly_yew::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::JsString;
use web_sys::js_sys::Promise;
use web_sys::js_sys::Reflect;
use web_sys::window;
use web_sys::HtmlElement;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ManualInterventionProps {
    pub children: Children,
    pub script: String,
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(ManualIntervention)]
pub fn manual_intervention(props: &ManualInterventionProps) -> Html {
    let code_block_ref = NodeRef::default();
    let button_text = use_state(|| "📋".to_string());
    let script_content = use_state(|| None::<ScriptEntry>);
    let loading = use_state(|| true);

    {
        let script = props.script.clone();
        let reload_trigger = props.reload_trigger;
        let script_content = script_content.clone();
        let loading = loading.clone();

        use_effect_with(reload_trigger, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let url = format!("/api/workstation/command/{}", script);
                let response = match gloo_net::http::Request::get(&url).send().await {
                    Ok(response) => response,
                    Err(_) => {
                        error!("Failed to fetch script");
                        return;
                    }
                };
                match response.json::<ScriptEntry>().await {
                    Ok(script) => {
                        script_content.set(Some(script));
                    }
                    Err(e) => {
                        error!(format!("Failed to deserialize script: {:?}", e));
                    }
                }
                loading.set(false);
            });

            || ()
        });
    }

    let copy_code = {
        let code_block_ref = code_block_ref.clone();
        let button_text = button_text.clone();
        Callback::from(move |_| {
            if let Some(element) = code_block_ref.cast::<HtmlElement>() {
                if let Some(content_element) = element.query_selector(".content").unwrap() {
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

                                let set_button_text = button_text.clone();
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
    };

    if *loading {
        html! {
            <LoadingState />
        }
    } else if let Some(script_entry) = (*script_content).clone() {
        html! {
            <div class="manual_intervention">
                <div class="command_area" style="position: relative;">
                    <div class="header">
                        {"👷 Manual intervention required"}
                    </div>
                    <Stack gutter=true>
                        <StackItem>
                            { for props.children.iter() }
                        </StackItem>
                        <StackItem>
                            <div class="code_container" ref={code_block_ref.clone()}>
                                <div class="content">
                                    <CodeBlock>
                                        <CodeBlockCode>
                                            {script_entry.script.clone()}
                                        </CodeBlockCode>
                                    </CodeBlock>
                                </div>
                                <button title="Copy script" class="copy-button" onclick={copy_code}><div class="copy-button-text">{ (*button_text).clone() }</div></button>
                            </div>
                        </StackItem>
                    </Stack>
                </div>
            </div>
        }
    } else {
        html! {
            <p>{ "Failed to load script." }</p>
        }
    }
}
