use crate::components::ButtonLink;
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
                        html! { <>
                                 <h1>{ format!("🖖 Welcome {}", workstation.user.name) }</h1>
                                </>
                        }
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

                        let distro_term = "Distribution";
                        let variant_text = match workstation.clone().platform.release.variant.as_str() {
                            "" => "".to_string(),
                            v => format!("({})", v.to_string().trim_matches('"'))
                        };
                        html! {
                            <>
                                <DescriptionList>
                                <DescriptionGroup term="🖥️ Workstation">
                                <code>{workstation.clone().hostname}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term={os_term}>
                                <code>{format!("{} {}", os_type, workstation.clone().platform.version.to_string())}</code>
                                </DescriptionGroup>
                                <DescriptionGroup term={distro_term}>
                                <code>{format!("{} {} {}", workstation.clone().platform.release.name, workstation.clone().platform.release.version, variant_text)}</code>
                                </DescriptionGroup>
                                </DescriptionList>
                                <hr/>
                                <p>
                            {"The Workstation screen consists of several tabs you should go through in order:"}
                            </p>
                                <ul>
                                <li>{"Install dependencies."}</li>
                                <li>{"Install d.rymcg.tech."}</li>
                                <li>{"Setup Docker contexts."}</li>
                                </ul>
                                </>
                        }
                    } else {
                        html! { <p>{ "Loading..." }</p> }
                    }
                }
        </CardBody>
            <CardFooter>
            <h1>{"Next:"}</h1>
            <ButtonLink href="/workstation#dependencies">{"⭐️ Install dependencies"}</ButtonLink>
            </CardFooter>
        </Card>
    }
}
