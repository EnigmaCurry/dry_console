use patternfly_yew::prelude::*;
use yew::prelude::*;

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::WebSocket;

pub struct TerminalOutput {
    messages: Vec<String>,
    ws: Option<WebSocket>,
    callback: Option<Closure<dyn FnMut(web_sys::MessageEvent)>>,
}

pub enum Msg {
    NewMessage(String),
    ConnectWebSocket,
}

impl Component for TerminalOutput {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            messages: Vec::new(),
            ws: None,
            callback: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NewMessage(message) => {
                self.messages.push(message);
                true
            }
            Msg::ConnectWebSocket => {
                let ws = WebSocket::new("/api/workstation/command_execute").unwrap();

                let callback = {
                    let link = ctx.link().clone();
                    Closure::wrap(Box::new(move |e: web_sys::MessageEvent| {
                        if let Some(text) = e.data().as_string() {
                            link.send_message(Msg::NewMessage(text));
                        }
                    }) as Box<dyn FnMut(_)>)
                };

                ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
                self.ws = Some(ws);
                self.callback = Some(callback);
                false
            }
        }
    }

    fn changed(&mut self, _ctx: &Context<Self>, _props: &Self::Properties) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { for self.messages.iter().map(|message| html!{ <p>{message}</p> }) }
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::ConnectWebSocket);
        }
    }
}
