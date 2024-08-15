use crate::components::ButtonLink;
use crate::pages::workstation::WorkstationTab;
use gloo::net::http::Request;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VChild;

#[derive(Clone, Deserialize)]
struct WorkstationDependencySpec {
    name: String,
    version: String,
}

impl WorkstationDependencySpec {
    fn get_dependency(&self) -> WorkstationDependency {
        WorkstationDependency {
            name: self.name.clone(),
            installed: None,
            version: self.version.clone(),
            path: "".to_string(),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
struct WorkstationDependency {
    name: String,
    installed: Option<bool>,
    version: String,
    path: String,
}

impl WorkstationDependency {
    // fn new(name: &str) -> Self {
    //     Self {
    //         name: name.to_string(),
    //         installed: None,
    //         path: "".to_string(),
    //         version: "*".to_string(),
    //     }
    // }

    async fn get_installed_state(&mut self) -> Result<WorkstationDependency, anyhow::Error> {
        let url = format!("/api/workstation/dependency/{}", self.name);
        let response = Request::get(&url).send().await?;
        let json_value: serde_json::Value = response.json().await?;
        let dependency: WorkstationDependency = serde_json::from_value(json_value)?;
        Ok(dependency)
    }

    fn validate(self) -> Option<bool> {
        self.installed?;
        if !self.installed.unwrap_or(false) || self.path.is_empty() || self.version.is_empty() {
            return Some(false);
        }
        Some(true)
    }
}

#[derive(Properties, PartialEq)]
pub struct DependencyListProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(DependencyList)]
pub fn dependency_list(props: &DependencyListProps) -> Html {
    let dependencies = use_state(Vec::new);
    let first_uninstalled = use_state(String::new);
    let status_checked = use_state(|| false);
    let is_loading = use_state(|| true);
    let has_fetched = use_state(|| false);
    let all_installed = use_state(|| false); // New state for summary message

    let fetch_dependencies = {
        let dependencies = dependencies.clone();
        let first_uninstalled = first_uninstalled.clone();
        let status_checked = status_checked.clone();
        let is_loading = is_loading.clone();
        let has_fetched = has_fetched.clone();
        let all_installed = all_installed.clone();
        let selected_tab = props.selected_tab.clone(); // Clone the selected_tab

        Callback::from(move |_| {
            if *has_fetched || selected_tab != WorkstationTab::Dependencies {
                return;
            }

            let dependencies = dependencies.clone();
            let first_uninstalled = first_uninstalled.clone();
            let status_checked = status_checked.clone();
            let is_loading = is_loading.clone();
            let has_fetched = has_fetched.clone();
            let all_installed = all_installed.clone();

            is_loading.set(true);
            status_checked.set(false);
            has_fetched.set(true);

            spawn_local(async move {
                match gloo_net::http::Request::get("/api/workstation/dependencies")
                    .send()
                    .await
                {
                    Ok(response) => {
                        let text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "Failed to get response text".to_string());

                        if let Ok(mut deps) =
                            serde_json::from_str::<Vec<WorkstationDependencySpec>>(&text)
                        {
                            let mut workstation_deps: Vec<WorkstationDependency> = Vec::new();
                            let mut all_installed_temp = true; // Temporary flag for checking installation status

                            for dep in deps.iter_mut() {
                                let mut dep = dep.get_dependency();
                                match dep.get_installed_state().await {
                                    Ok(state) => {
                                        dep = state;
                                    }
                                    Err(_e) => {}
                                }
                                if !dep.clone().validate().unwrap_or(false) {
                                    all_installed_temp = false;
                                }
                                workstation_deps.push(dep);
                            }

                            if let Some(dep) = deps
                                .iter()
                                .find(|dep| dep.get_dependency().installed == Some(false))
                            {
                                first_uninstalled.set(dep.name.clone());
                            }

                            all_installed.set(all_installed_temp); // Update the all_installed state

                            dependencies.set(workstation_deps);
                        } else {
                            log::error!("Failed to parse dependencies response");
                        }
                    }
                    Err(_) => {
                        log::error!("Failed to fetch dependencies");
                    }
                }
                status_checked.set(true);
                is_loading.set(false);
            });
        })
    };

    {
        let fetch_dependencies = fetch_dependencies.clone();
        let _ = props.reload_trigger; // Use the prop to trigger re-fetching

        use_effect(move || {
            fetch_dependencies.emit(());
            || ()
        });
    }

    let toggle = {
        let first_uninstalled = first_uninstalled.clone();
        Callback::from(move |key: String| {
            first_uninstalled.set(key);
        })
    };

    let accordion_items = dependencies
        .iter()
        .map(|dep| {
            let title = match dep.clone().validate() {
                Some(true) => format!("✅ {}", dep.name),
                Some(false) => format!("⚠️ {}", dep.name),
                None => format!("⏳️ {}", dep.name),
            };

            let on_toggle = {
                let name = dep.name.clone();
                let toggle = toggle.clone();
                Callback::from(move |_| toggle.emit(name.clone()))
            };

            let is_expanded = *first_uninstalled == dep.name;

            html_nested! {
                <AccordionItem title={title} expanded={is_expanded} onclick={on_toggle}>
                    <div>
                { match dep.installed {
                    None => html! {"Dependency check is pending ..."},
                    Some(false) => {
                        html! { format!("{} is not installed", dep.name) }
                    }
                    Some(true) => {//
                        match dep.clone().validate() {
                            Some(is_valid) => {
                                match is_valid {
                                    true => {
                                        html! {
                                            <DescriptionList>
                                                <DescriptionGroup term="Path">
                                                <code>
                                            { dep.path.to_string() }
                                            </code>
                                                </DescriptionGroup>
                                                <DescriptionGroup term="Version">
                                                <code>
                                            { dep.version.to_string() }
                                            </code>
                                                </DescriptionGroup>
                                                </DescriptionList>
                                        }
                                    },
                                    false => {
                                        if dep.clone().path.is_empty() {
                                            html! {"Validation error: path is empty"}
                                        } else if dep.clone().version.is_empty() {
                                            html! {"Validation error: version is empty"}
                                        } else {
                                            html! {"Validation error"}
                                        }
                                    }
                                }

                            },
                            None => {html! {"Dependency check is pending ..."}}
                        }
                    }
                }}
                </div>
                </AccordionItem>
            }
        })
        .collect::<Vec<VChild<AccordionItem>>>();

    let on_click = {
        let fetch_dependencies = fetch_dependencies.clone();
        let has_fetched = has_fetched.clone();
        Callback::from(move |_: MouseEvent| {
            has_fetched.set(false);
            fetch_dependencies.emit(());
        })
    };

    html! {
        <>
            if *is_loading {
                <Card>
                    <CardTitle><p><h1>{"⌛️ Checking dependencies, please wait ..."}</h1></p></CardTitle>
                    <CardBody>
                    <div class="flex-center">
                    <Spinner size={SpinnerSize::Custom(String::from("80px"))} aria_label="Contents of the custom size example" />
                    </div>
                    </CardBody>
                    </Card>
                } else {
                <Card>
                    <CardTitle>
                    <div >
                    <span>
                    { if *all_installed {
                        html! { <p><h1> {"😎 All dependencies found!"} </h1></p> }
                    } else {
                        html! { <p><h1> {"⁉️ Warning!"} </h1> {"Not all workstation dependencies were found. Please install all the dependencies before proceeding."} </p>}
                    } }
                    </span>
                    <br/>
                    <Button label="🔄 Recheck dependencies" onclick={on_click} />
                    <br/>
                    </div>
                    </CardTitle>
                    <CardBody>
                    <Accordion>
                { accordion_items }
                </Accordion>
                    </CardBody>
                    <CardFooter>
                    <h1>{"Next:"}</h1>
                    <ButtonLink href="/workstation#d-rymcg-tech">{"⭐️ Install d.rymcg.tech"}</ButtonLink>
                    </CardFooter>
                    </Card>
            }
        </>
    }
}
