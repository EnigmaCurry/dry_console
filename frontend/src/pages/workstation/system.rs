use gloo_net::http::Request;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use yew::prelude::*;

#[derive(Deserialize, Debug, Clone)]
struct User {
    #[allow(dead_code)]
    uid: u32,
    name: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Workstation {
    hostname: String,
    user: User,
}

#[function_component(System)]
pub fn system() -> Html {
    let workstation = use_state(|| None);
    let workstation_clone = workstation.clone();

    wasm_bindgen_futures::spawn_local(async move {
        if workstation_clone.is_none() {
            let fetched_data: Workstation = Request::get("/api/workstation/")
                .send()
                .await
                .expect("Failed to fetch data")
                .json()
                .await
                .expect("Failed to parse JSON");
            workstation_clone.set(Some(fetched_data));
        }
    });

    html! {
        <Card>
            <CardTitle>
                {
                    if let Some(workstation) = &*workstation {
                        html! { <h1>{ format!("🖖 Welcome {}", workstation.user.name) }</h1> }
                    } else {
                        html! { <h1>{ "🖖 Welcome" }</h1> }
                    }
                }
            </CardTitle>
            <CardBody>
                {
                    if let Some(workstation) = &*workstation {
                        html! {
                            <>
                                <p>{ "Workstation 🖥️ : " }<code>{workstation.clone().hostname}</code></p>
                                <br/>
                                <p>{ "Todo items:" }</p>
                                <ol>
                                    <li><a href="/workstation#dependencies">{ "Install dependencies" }</a></li>
                                </ol>
                            </>
                        }
                    } else {
                        html! { <p>{ "Loading..." }</p> }
                    }
                }
            </CardBody>
        </Card>
    }
}
