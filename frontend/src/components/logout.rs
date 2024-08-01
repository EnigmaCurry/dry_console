use crate::app::{AppRoute, SessionState};
use gloo_net::http::Request;
use patternfly_yew::prelude::*;
use std::{rc::Rc, sync::Arc, time::Duration};
use web_sys::SubmitEvent;
use yew::prelude::*;
use yew_nested_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LogoutProps {
    pub session_state: UseStateHandle<crate::app::SessionState>,
}

#[function_component(Logout)]
pub fn logout(props: &LogoutProps) -> Html {
    let toaster = use_toaster().expect("Must be nested inside a ToastViewer");
    let loading_state = use_state(|| false);

    let session_state = props.session_state.clone();
    let router = use_router().unwrap();
    let toast = Arc::new({
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
                        router.push(AppRoute::Host);
                    }
                    _ => {
                        toast(AlertType::Danger, "Logout error!");
                    }
                }
                loading_state.set(false);
            });
        })
    };

    html! {
        <>
            if *loading_state {
                <div>{"Loading state ..."}</div>
            } else {
                if (*session_state).logged_in {
                    <div>
                        <form onsubmit={logout_submit}>
                            <Button label="Logout" r#type={ButtonType::Submit} />
                        </form>
                    </div>
                } else {
                      <div>{"Not logged in."}</div>
                }
            }
        </>
    }
}
