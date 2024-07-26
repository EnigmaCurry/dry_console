use anyhow::Error;
use gloo_net::http::Request;
use serde::Serialize;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct LoginForm {
    token: String,
}

pub enum Msg {
    UpdateToken(String),
    Submit,
    Response(Result<String, Error>),
}

#[derive(Serialize)]
struct LoginRequest {
    token: String,
}

impl Component for LoginForm {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            token: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateToken(token) => {
                self.token = token;
                true
            }
            Msg::Submit => {
                let request = LoginRequest {
                    token: self.token.clone(),
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
                    <label for="token">{ "Token: " }</label>
                    <input
                        id="token"
                        type="text"
                        name="token"
                        autocomplete="one-time-code"
                        value={self.token.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {
                            let input: HtmlInputElement = e.target_unchecked_into();
                            Msg::UpdateToken(input.value())
                        })}
                    />
                </div>
                <button type="submit">{ "Login" }</button>
            </form>
        }
    }
}
