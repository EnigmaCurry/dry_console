use anyhow::Error;
use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Clone)]
pub struct Notification {
    pub messages: Vec<String>,
}

pub struct NotificationComponent {
    messages: Vec<String>,
}

pub enum Msg {
    FetchMessages,
    SetMessages(Result<Notification, Error>),
}

impl Component for NotificationComponent {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::FetchMessages); // Fetch messages when component is created
        Self {
            messages: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchMessages => {
                let callback = ctx
                    .link()
                    .callback(|response: Result<Notification, Error>| Msg::SetMessages(response));

                wasm_bindgen_futures::spawn_local(async move {
                    let response = Request::get("/api/session/messages").send().await;

                    let result = match response {
                        Ok(resp) => match resp.json::<Notification>().await {
                            Ok(notification) => Ok(notification),
                            Err(err) => Err(Error::new(err)),
                        },
                        Err(err) => Err(Error::new(err)),
                    };

                    callback.emit(result);
                });

                false
            }
            Msg::SetMessages(response) => {
                match response {
                    Ok(notification) => {
                        self.messages = notification.messages;
                    }
                    Err(error) => {
                        web_sys::console::log_1(&format!("Error: {:?}", error).into());
                    }
                }
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="notification-area">
                { for self.messages.iter().map(|message| html! {
                    <div class="notification">{ message }</div>
                }) }
            </div>
        }
    }
}
