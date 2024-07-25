use anyhow::Error;
use gloo_net::http::Request;
use serde::Serialize;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct LoginForm {
    username: String,
    password: String,
    next_url: String,
}

pub enum Msg {
    UpdateUsername(String),
    UpdatePassword(String),
    Submit,
    Response(Result<String, Error>),
}

#[derive(Serialize)]
struct LoginRequest {
    username: String,
    password: String,
    next: String,
}

impl Component for LoginForm {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            next_url: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateUsername(username) => {
                self.username = username;
                true
            }
            Msg::UpdatePassword(password) => {
                self.password = password;
                true
            }
            Msg::Submit => {
                let request = LoginRequest {
                    username: self.username.clone(),
                    password: self.password.clone(),
                    next: self.next_url.clone(),
                };

                let callback = ctx
                    .link()
                    .callback(|response: Result<String, Error>| Msg::Response(response));

                wasm_bindgen_futures::spawn_local(async move {
                    let response = Request::post("/api/session/login")
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&request).unwrap())
                        .expect("Failed to build request.")
                        .send()
                        .await;

                    let result = match response {
                        Ok(resp) => Ok(resp.text().await.unwrap()),
                        Err(err) => Err(Error::new(err)),
                    };

                    callback.emit(result);
                });

                true
            }
            Msg::Response(response) => {
                match response {
                    Ok(body) => {
                        web_sys::console::log_1(&body.into());
                    }
                    Err(error) => {
                        web_sys::console::log_1(&format!("Error: {:?}", error).into());
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <form onsubmit={ctx.link().callback(|e: SubmitEvent| {
                e.prevent_default();
                Msg::Submit
            })}>
                <div>
                    <label for="username">{ "Username: " }</label>
                    <input
                        type="text"
                        id="username"
                        value={self.username.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateUsername(input.value())
                        })}
                    />
                </div>
                <div>
                    <label for="password">{ "Password: " }</label>
                    <input
                        type="password"
                        id="password"
                        value={self.password.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdatePassword(input.value())
                        })}
                    />
                </div>
                <button type="submit">{ "Login" }</button>
            </form>
        }
    }
}
