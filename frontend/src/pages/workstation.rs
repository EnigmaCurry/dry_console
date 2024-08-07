use anyhow::anyhow;
use gloo::net::http::Request;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VChild;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkstationTab {
    Dependencies,
}

#[function_component(WorkstationTabs)]
fn tabs() -> Html {
    let selected = use_state_eq(|| WorkstationTab::Dependencies);
    let onselect = use_callback(selected.clone(), |index, selected| selected.set(index));

    html! (
        <>
            <Tabs<WorkstationTab> detached=true {onselect} selected={*selected} r#box=true>
                <Tab<WorkstationTab> index={WorkstationTab::Dependencies} title="Dependencies"/>
            </Tabs<WorkstationTab>>
            <section hidden={(*selected) != WorkstationTab::Dependencies}>
                <DependencyList/>
            </section>
        </>
    )
}

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
        if self.installed == None {
            return None;
        }
        if !self.installed.unwrap_or(false) || self.path.is_empty() || self.version.is_empty() {
            return Some(false);
        }
        Some(true)
    }
}

#[function_component(DependencyList)]
fn dependency_list() -> Html {
    let dependencies = use_state(Vec::new);
    let first_uninstalled = use_state(|| String::new());
    let status_checked = use_state(|| false);
    let is_loading = use_state(|| true);
    let has_fetched = use_state(|| false);

    let fetch_dependencies = {
        let dependencies = dependencies.clone();
        let first_uninstalled = first_uninstalled.clone();
        let status_checked = status_checked.clone();
        let is_loading = is_loading.clone();
        let has_fetched = has_fetched.clone();

        Callback::from(move |_| {
            if *has_fetched {
                return;
            }

            let dependencies = dependencies.clone();
            let first_uninstalled = first_uninstalled.clone();
            let status_checked = status_checked.clone();
            let is_loading = is_loading.clone();
            let has_fetched = has_fetched.clone();

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
                            for dep in deps.iter_mut() {
                                let mut dep = dep.get_dependency();
                                match dep.get_installed_state().await {
                                    Ok(state) => {
                                        dep = state;
                                    }
                                    Err(_e) => {}
                                }
                                workstation_deps.push(dep);
                            }

                            if let Some(dep) = deps
                                .iter()
                                .find(|dep| dep.get_dependency().installed == Some(false))
                            {
                                first_uninstalled.set(dep.name.clone());
                            }

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
        .enumerate()
        .map(|(_index, dep)| {
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
                    <CardTitle>{"Checking dependencies, please wait ..."}</CardTitle>
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
                    <Button label="Recheck dependencies" onclick={on_click} />
                    </div>
                    </CardTitle>
                    <CardBody>
                    <Accordion>
                { accordion_items }
                </Accordion>
                    </CardBody>
                    </Card>
            }
        </>
    }
}

#[function_component(Workstation)]
pub fn workstation() -> Html {
    html! {
        <PageSection>
            <WorkstationTabs/>
        </PageSection>
    }
}
