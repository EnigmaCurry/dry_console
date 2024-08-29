use crate::components::manual_intervention::ManualIntervention;
use crate::components::terminal::TerminalOutput;
use crate::components::ButtonLink;
use crate::pages::workstation::{SystemInfoContext, WorkstationTab};
use anyhow::anyhow;
use dry_console_dto::workstation::WorkstationPackage;
use gloo::console::debug;
use gloo::net::http::Request;
use itertools::Itertools;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VChild;

use super::SystemInfo;

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
            packages: Vec::<WorkstationPackage>::new(),
        }
    }
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
struct WorkstationDependency {
    name: String,
    installed: Option<bool>,
    version: String,
    path: String,
    packages: Vec<WorkstationPackage>,
}
impl WorkstationDependency {
    async fn get_installed_state(&mut self) -> Result<WorkstationDependency, anyhow::Error> {
        let url = format!("/api/workstation/dependency/{}/", self.name);
        let response;
        let json_value: serde_json::Value;
        match Request::get(&url).send().await {
            Ok(r) => response = r,
            Err(e) => return Err(anyhow!("one: {}", e)),
        };
        match response.json().await {
            Ok(r) => json_value = r,
            Err(e) => return Err(anyhow!("two: {}", e)),
        };
        //debug!(format!("json_value: {:?}", json_value));
        match serde_json::from_value(json_value) {
            Ok(j) => Ok(j),
            Err(e) => Err(anyhow!("three: {}", e)),
        }
    }

    fn validate(self) -> Option<bool> {
        self.installed?;
        if !self.installed.unwrap_or(false) || self.path.is_empty() || self.version.is_empty() {
            return Some(false);
        }
        Some(true)
    }
}

#[derive(Properties, PartialEq, Clone)]
struct DependencyItemProps {
    dependency: WorkstationDependency,
    is_expanded: bool,
    on_toggle: Callback<String>,
}
#[function_component(DependencyItem)]
fn dependency_item(props: &DependencyItemProps) -> Html {
    let title = match props.dependency.clone().validate() {
        Some(true) => format!("✅ {}", props.dependency.name),
        Some(false) => format!("⚠️ {}", props.dependency.name),
        None => format!("⏳️ {}", props.dependency.name),
    };

    let on_toggle = {
        let name = props.dependency.name.clone();
        let toggle = props.on_toggle.clone();
        Callback::from(move |_| toggle.emit(name.clone()))
    };

    html_nested! {
        <AccordionItem title={title} expanded={props.is_expanded} onclick={on_toggle}>
            <div>
                { match props.dependency.installed {
                    None => html! {"Dependency check is pending ..."},
                    Some(false) => {
                        html! { format!("{} is not installed", props.dependency.name) }
                    }
                    Some(true) => {//
                        match props.dependency.clone().validate() {
                            Some(is_valid) => {
                                match is_valid {
                                    true => {
                                        html! {
                                            <DescriptionList>
                                                <DescriptionGroup term="Path">
                                                    <code>
                                                    { props.dependency.path.to_string() }
                                                    </code>
                                                </DescriptionGroup>
                                                <DescriptionGroup term="Version">
                                                    <code>
                                                    { props.dependency.version.to_string() }
                                                    </code>
                                                </DescriptionGroup>
                                            </DescriptionList>
                                        }
                                    },
                                    false => {
                                        if props.dependency.clone().path.is_empty() {
                                            html! {"Validation error: path is empty"}
                                        } else if props.dependency.clone().version.is_empty() {
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
    .into() // Convert the VChild into a VNode
}

#[function_component(LoadingState)]
fn loading_state() -> Html {
    html! {
        <Card>
            <CardTitle><p><h1>{"⌛️ Checking dependencies, please wait ..."}</h1></p></CardTitle>
            <CardBody>
                <div class="flex-center">
                    <Spinner size={SpinnerSize::Custom(String::from("80px"))} aria_label="Contents of the custom size example" />
                </div>
            </CardBody>
        </Card>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct DependencySummaryProps {
    all_installed: bool,
    on_recheck: Callback<MouseEvent>,
}
#[function_component(DependencySummary)]
fn dependency_summary(props: &DependencySummaryProps) -> Html {
    html! {
        <CardTitle>
            <div>
                <span>
                    { if props.all_installed {
                        html! { <p><h1> {"😎 All dependencies found!"} </h1></p> }
                    } else {
                        html! { <p><h1> {"⁉️ Warning!"} </h1> {"Not all workstation dependencies were found. Please install all the dependencies before proceeding."} </p>}
                    } }
                </span>
            </div>
        </CardTitle>
    }
}

fn create_fetch_dependencies_callback(
    selected_tab: WorkstationTab,
    dependencies: UseStateHandle<Vec<WorkstationDependency>>,
    first_uninstalled: UseStateHandle<String>,
    is_loading: UseStateHandle<bool>,
    has_fetched: UseStateHandle<bool>,
    all_installed: UseStateHandle<bool>,
    uninstalled_dependencies: UseStateHandle<Vec<WorkstationDependency>>,
) -> Callback<()> {
    Callback::from(move |_| {
        if *has_fetched || selected_tab != WorkstationTab::Dependencies {
            return;
        }

        let dependencies = dependencies.clone();
        let first_uninstalled = first_uninstalled.clone();
        let is_loading = is_loading.clone();
        let has_fetched = has_fetched.clone();
        let all_installed = all_installed.clone();
        let uninstalled_dependencies = uninstalled_dependencies.clone();

        is_loading.set(true);
        has_fetched.set(true);

        spawn_local(async move {
            match gloo_net::http::Request::get("/api/workstation/dependencies/")
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
                        let mut uninstalled_deps: Vec<WorkstationDependency> = Vec::new();
                        let mut all_installed_temp = true;

                        for dep in deps.iter_mut() {
                            let mut dep = dep.get_dependency();
                            match dep.get_installed_state().await {
                                Ok(state) => {
                                    dep = state;
                                }
                                Err(e) => {
                                    panic!("{}", e);
                                }
                            }
                            if !dep.clone().validate().unwrap_or(false) {
                                all_installed_temp = false;
                                uninstalled_deps.push(dep.clone());
                            }
                            workstation_deps.push(dep);
                        }

                        if let Some(dep) = uninstalled_deps.first() {
                            first_uninstalled.set(dep.name.clone());
                        }

                        all_installed.set(all_installed_temp);
                        dependencies.set(workstation_deps);
                        uninstalled_dependencies.set(uninstalled_deps);
                    } else {
                        log::error!("Failed to parse dependencies response");
                    }
                }
                Err(_) => {
                    log::error!("Failed to fetch dependencies");
                }
            }
            is_loading.set(false);
        });
    })
}

fn create_accordion_items(
    dependencies: &[WorkstationDependency],
    first_uninstalled: &str,
    toggle: Callback<String>,
) -> Vec<VChild<AccordionItem>> {
    dependencies
        .iter()
        .map(|dep: &WorkstationDependency| {
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

            let is_expanded = first_uninstalled == dep.name;
            let packages_str = dep
                .packages
                .iter()
                .map(|pkg| pkg.package_name.as_str())
                .collect::<Vec<&str>>()
                .join(" ");

            html_nested! {
                <AccordionItem title={title} expanded={is_expanded} onclick={on_toggle}>
                    <div>
                        { match dep.installed {
                            None => html! {"Dependency check is pending ..."},
                            Some(false) => html! {
                                <DescriptionList>
                                    {format!("{} is not installed", dep.name)}
                                    <DescriptionGroup term="Packages required:">
                                    <code>{ packages_str }</code>
                                    </DescriptionGroup>
                                </DescriptionList>},
                            Some(true) => {
                                match dep.clone().validate() {
                                    Some(true) => html! {
                                        <DescriptionList>
                                            <DescriptionGroup term="Path">
                                                <code>{ dep.path.clone() }</code>
                                            </DescriptionGroup>
                                            <DescriptionGroup term="Version">
                                                <code>{ dep.version.clone() }</code>
                                            </DescriptionGroup>
                                        </DescriptionList>
                                    },
                                    Some(false) => {
                                        if dep.path.is_empty() {
                                            html! { "Validation error: path is empty" }
                                        } else if dep.version.is_empty() {
                                            html! { "Validation error: version is empty" }
                                        } else {
                                            html! { "Validation error" }
                                        }
                                    },
                                    None => html! { "Dependency check is pending ..." }
                                }
                            }
                        }}
                    </div>
                </AccordionItem>
            } // Convert VChild<AccordionItem> to VNode
        })
        .collect()
}

#[derive(Properties, PartialEq)]
pub struct DependencyListProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
    pub system_info: SystemInfo,
}
#[function_component(DependencyList)]
pub fn dependency_list(props: &DependencyListProps) -> Html {
    let system_info_context = use_context::<SystemInfoContext>();
    let dependencies = use_state(Vec::<WorkstationDependency>::new);
    let first_uninstalled = use_state(String::new);
    let is_loading = use_state(|| true);
    let has_fetched = use_state(|| false);
    let all_installed = use_state(|| false);
    let uninstalled_dependencies = use_state(|| Vec::<WorkstationDependency>::new());

    let fetch_dependencies = create_fetch_dependencies_callback(
        props.selected_tab.clone(),
        dependencies.clone(),
        first_uninstalled.clone(),
        is_loading.clone(),
        has_fetched.clone(),
        all_installed.clone(),
        uninstalled_dependencies.clone(),
    );

    // Effect to fetch dependencies when `has_fetched` is reset to false
    {
        let has_fetched = has_fetched.clone();
        let fetch_dependencies = fetch_dependencies.clone();

        use_effect(move || {
            if !*has_fetched {
                fetch_dependencies.emit(());
            }
            || ()
        });
    }

    // on_click only resets the `has_fetched` state
    let on_click = {
        let has_fetched = has_fetched.clone();
        let first_uninstalled = first_uninstalled.clone();
        Callback::from(move |_: MouseEvent| {
            has_fetched.set(false);
            first_uninstalled.set("".to_string());
        })
    };

    if *is_loading {
        return html! { <LoadingState /> };
    }

    let toggle = {
        let first_uninstalled = first_uninstalled.clone();
        Callback::from(move |key: String| {
            first_uninstalled.set(key);
        })
    };

    let accordion_items = create_accordion_items(&dependencies, &first_uninstalled, toggle);

    let uninstalled_list = uninstalled_dependencies
        .iter()
        .flat_map(|dep| dep.packages.iter())
        .map(|pkg| pkg.package_name.clone())
        .collect::<HashSet<String>>()
        .into_iter()
        .collect::<Vec<String>>()
        .into_iter()
        .sorted()
        .collect::<Vec<String>>()
        .join(" ");

    html! {
        <>
            <Card>
                <DependencySummary all_installed={*all_installed} on_recheck={on_click.clone()} />
                <CardBody>
                if !*all_installed {
                    if props.system_info.user.can_sudo {
                        <TerminalOutput script="InstallDependencies" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={on_click.clone()}/>
                            <br/>
                            <Button label="🔄 Recheck dependencies" onclick={on_click.clone()} />
                            <br/>
                    } else {
                        <ManualIntervention script={format!("sudo dnf install -y {}", uninstalled_list)} reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()}>
                            <h2>{"Root privileges are required to install missing packages."}</h2>
                            <p>{"You may fix this condition by restarting dry_console with the "}<code>{"--sudo"}</code>{" argument, or you may manually run the following commands by copy and pasting them into your workstation terminal."}</p>
                        <br/>
                            <ul>
                            <li>{"Click the clipboard button to copy the script below."}</li>
                            <li>{"Open your workstation's terminal application."}</li>
                            <li>{"Paste the script into the terminal and press Enter to run it."}</li>
                            <li>{"Check for any authentication prompt and enter your credentials."}</li>
                            <li>{"Click the "}<code>{"Recheck dependencies"}</code>{" button once complete."}</li>
                            </ul>
                            <br/>
                            <Button label="🔄 Recheck dependencies" onclick={on_click.clone()}/>
                            <br/>
                        </ManualIntervention>
                    }
                }
                    <Accordion>
                        { accordion_items }
                    </Accordion>
                </CardBody>
                if *all_installed {
                    <CardFooter>
                        <h1>{"Next:"}</h1>
                        <ButtonLink href="/workstation#d-rymcg-tech">{"⭐️ Install d.rymcg.tech"}</ButtonLink>
                    </CardFooter>
                }
            </Card>
        </>
    }
}
