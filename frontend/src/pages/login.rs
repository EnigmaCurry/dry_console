use crate::app::{AppRoute, SessionState};
use gloo::{utils::window};
use gloo_net::http::Request;
use patternfly_yew::prelude::*;
use serde::Serialize;
use std::{rc::Rc, time::Duration};
use web_sys::{HtmlInputElement, SubmitEvent};
use yew::prelude::*;
use yew_nested_router::prelude::*;

#[derive(Serialize)]
struct LoginData {
    token: String,
}

#[derive(Properties, PartialEq)]
pub struct LoginProps {
    pub session_state: UseStateHandle<crate::app::SessionState>,
}

#[function_component(Login)]
pub fn login(props: &LoginProps) -> Html {
    let toaster = use_toaster().expect("Must be nested inside a ToastViewer");
    let token_state = use_state(|| None::<String>);
    let loading_state = use_state(|| false);

    let session_state = props.session_state.clone();
    let router = use_router().unwrap();
    let toast = Rc::new({
        let toaster = toaster.clone();
        move |t: AlertType, msg: &str| {
            toaster.toast(Toast {
                title: msg.into(),
                timeout: Some(Duration::from_secs(match t {
                    AlertType::Danger => 5,
                    _ => 2,
                })),
                r#type: t,
                ..Default::default()
            });
        }
    });

    {
        let session_state = session_state.clone();
        let loading_state = loading_state.clone();
        let router = router.clone();
        let toast = toast.clone();
        use_effect_with((), move |_| {
            let location = window().location();
            if let Ok(hash) = location.hash() {
                if hash.starts_with("#token:") {
                    let token = hash.trim_start_matches("#token:").to_string();
                    loading_state.set(true);

                    let session_state = session_state.clone();
                    let router = router.clone();
                    let toast = toast.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let login_data = LoginData { token };
                        let response = Request::post("/api/session/login/")
                            .header("Content-Type", "application/json")
                            .json(&login_data)
                            .expect("Failed to serialize request")
                            .send()
                            .await;

                        match response {
                            Ok(res) if res.ok() => {
                                session_state.set(SessionState {
                                    logged_in: true,
                                    new_login_allowed: false,
                                });
                                toast(AlertType::Success, "Login successful!");
                                router.push(AppRoute::Workstation);
                            }
                            _ => {
                                toast(AlertType::Warning, "Login failed.");
                            }
                        }
                        loading_state.set(false);
                    });
                }
            }
            || ()
        });
    }

    let login_submit = {
        let token_state = Rc::new(token_state.clone());
        let loading_state = Rc::new(loading_state.clone());
        let session_state = Rc::new(session_state.clone());
        let router = router.clone();
        let toast = toast.clone();
        Callback::from(move |e: SubmitEvent| {
            let toast = toast.clone();
            e.prevent_default();
            match &**Rc::clone(&token_state).as_ref() {
                None => {
                    toast(
                        AlertType::Warning,
                        "You must enter a token before logging in.",
                    );
                }
                Some(token) => {
                    let token_clone = token.clone();
                    let loading_state_clone = loading_state.clone();
                    let session_state_clone = session_state.clone();
                    let router_clone = router.clone();

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
                                toast(AlertType::Success, "Login successful!");
                                session_state_clone.set(SessionState {
                                    logged_in: true,
                                    new_login_allowed: false,
                                });
                                router_clone.push(AppRoute::Workstation); // Redirect to index after successful login
                            }
                            Ok(r) => match r.status() {
                                401 => toast(AlertType::Warning, "Invalid token!"),
                                503 => toast(AlertType::Danger, "Login disabled!"),
                                _ => toast(AlertType::Danger, "Login error!"),
                            },
                            Err(_) => {
                                toast(AlertType::Danger, "Login failed!");
                            }
                        }
                        loading_state_clone.set(false);
                    });
                }
            }
        })
    };
    let logout_submit = {
        let loading_state = Rc::new(loading_state.clone());
        let session_state = Rc::new(session_state.clone());
        let router = Rc::new(router);
        Callback::from(move |e: SubmitEvent| {
            let toast = toast.clone();
            e.prevent_default();

            let loading_state = Rc::clone(&loading_state);
            loading_state.set(true);

            let session_state = Rc::clone(&session_state);
            let router = Rc::clone(&router);

            wasm_bindgen_futures::spawn_local(async move {
                let response = Request::post("/api/session/logout/").send().await;

                match response {
                    Ok(res) if res.ok() => {
                        toast(AlertType::Success, "Logged out!");
                        session_state.set(SessionState {
                            logged_in: false,
                            new_login_allowed: false,
                        });
                        router.push(AppRoute::Workstation);
                    }
                    _ => {
                        toast(AlertType::Danger, "Logout error!");
                    }
                }
                loading_state.set(false);
            });
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

    if session_state.logged_in {
        window().location().set_href("/").unwrap();
    }

    html! {
        <PageSection>
            if *loading_state {
                <div>{"Logging in..."}</div>
            } else {
                if session_state.logged_in {
                    <div>
                      <div>{"Already logged in."}</div>
                        <form onsubmit={logout_submit}>
                            <Button label="Logout" r#type={ButtonType::Submit} />
                        </form>
                    </div>
                } else if ! session_state.new_login_allowed {
                      <div>{"You are logged out. No new sessions are allowed at this time. (You must restart this service to create a new session)."}</div>
                } else {
                    <div>
                        <p>{"Login"}</p>
                        <form onsubmit={login_submit}>
                            <TextInput
                                r#type={TextInputType::Text}
                                placeholder="Enter your token"
                                value={token_value.clone()}
                                oninput={oninput}
                            />
                            <Button label="Login" r#type={ButtonType::Submit} />
                        </form>
                    </div>
                }
            }
        </PageSection>
    }
}
