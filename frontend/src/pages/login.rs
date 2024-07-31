use gloo::utils::window;
use gloo_net::http::Request;
use patternfly_yew::prelude::*;
use serde::Serialize;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, SubmitEvent};
use yew::prelude::*;

#[derive(Serialize)]
struct LoginData {
    token: String,
}

#[function_component(Login)]
pub fn login() -> Html {
    let token_state = use_state(|| None::<String>);
    let loading_state = use_state(|| false);

    {
        let token_state = token_state.clone();
        let loading_state = loading_state.clone();
        use_effect(move || {
            let location = window().location();
            if let Ok(hash) = location.hash() {
                if hash.starts_with("#token:") {
                    let token = hash.trim_start_matches("#token:").to_string();
                    token_state.set(Some(token.clone()));
                    loading_state.set(true);

                    // Perform the POST request immediately
                    let token_clone = token.clone();
                    let loading_state_clone = loading_state.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let login_data = LoginData { token: token_clone };
                        let response = Request::post("/api/session/login/")
                            .header("Content-Type", "application/json")
                            .json(&login_data)
                            .expect("Failed to serialize request")
                            .send()
                            .await;

                        match response {
                            Ok(res) if res.ok() => {
                                gloo::console::log!("Login successful!");
                            }
                            _ => {
                                gloo::console::error!("Login failed.");
                            }
                        }
                        loading_state_clone.set(false);
                    });
                }
            }
            || {}
        });
    }

    let onsubmit = {
        let token_state = token_state.clone();
        let loading_state = loading_state.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let Some(token) = token_state.as_ref() {
                let token_clone = token.clone();
                let loading_state_clone = loading_state.clone();

                loading_state.set(true);

                wasm_bindgen_futures::spawn_local(async move {
                    let login_data = LoginData { token: token_clone };
                    let response = Request::post("/api/session/login/")
                        .header("Content-Type", "application/json")
                        .json(&login_data)
                        .expect("Failed to serialize request")
                        .send()
                        .await;

                    match response {
                        Ok(res) if res.ok() => {
                            gloo::console::log!("Login successful!");
                        }
                        _ => {
                            gloo::console::error!("Login failed.");
                        }
                    }
                    loading_state_clone.set(false);
                });
            }
        })
    };

    let token_value = token_state.as_ref().unwrap_or(&"".to_string()).clone();

    let oninput = {
        let token_state = token_state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            token_state.set(Some(input.value()));
        })
    };

    html! {
        <PageSection>
            if *loading_state {
                <div>{"Logging in..."}</div>
            } else {
                if token_state.is_some() {
                    <div>{"Token found in URL, logging in automatically..."}</div>
                } else {
                    <div>
                        <p>{"Login"}</p>
                        <form {onsubmit}>
                            <TextInput
                                r#type={TextInputType::Text}
                                placeholder="Enter your token"
                                value={token_value.clone()}
                                oninput={oninput}
                            />
                            <Button label="Submit" r#type={ButtonType::Submit} />
                        </form>
                    </div>
                }
            }
        </PageSection>
    }
}
