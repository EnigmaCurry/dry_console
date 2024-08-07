use dry_console_dto::workstation::WorkstationState;
use gloo_net::http::Request;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[function_component(System)]
pub fn system() -> Html {
    let workstation = use_state(|| None);
    let workstation_clone = workstation.clone();

    wasm_bindgen_futures::spawn_local(async move {
        if workstation_clone.is_none() {
            let fetched_data: WorkstationState = Request::get("/api/workstation/")
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
                        let os_type = workstation.clone().platform.os_type.to_string();
                        let os_term = if os_type == "Linux" {
                            "🐧 OS Type"
                        } else if os_type == "MacOS" {
                            "🍎 OS Type"
                        } else if os_type == "Windows" {
                            "🤑 OS Type"
                        } else {
                            "OS Type"
                        };

                        html! {
                            <DescriptionList>
                                <DescriptionGroup term="🖥️ Workstation">
                                    <code>{workstation.clone().hostname}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term={os_term}>
                                    <code>{format!("{} {}", os_type, workstation.clone().platform.version.to_string())}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term="Dependencies">
                                    <a href="/workstation#dependencies">{ "Install dependencies" }</a>
                                </DescriptionGroup>
                            </DescriptionList>
                        }
                    } else {
                        html! { <p>{ "Loading..." }</p> }
                    }
                }
            </CardBody>
        </Card>
    }
}
