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

#[derive(Clone)]
struct WorkstationDependency {
    name: String,
    installed: Option<bool>,
    path: String,
}

#[derive(Deserialize)]
struct WorkstationDependencyStatus {
    installed: bool,
}

impl WorkstationDependency {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            installed: None,
            path: "".to_string(),
        }
    }

    async fn is_installed(&mut self) -> Result<bool, anyhow::Error> {
        let url = format!("/api/workstation/dependency/{}", self.name);
        match Request::get(&url).send().await {
            Ok(r) => {
                let status: WorkstationDependencyStatus = r.json().await?;
                Ok(status.installed)
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
}

#[function_component(DependencyList)]
fn dependency_list() -> Html {
    let dependencies = use_state(|| {
        vec![
            WorkstationDependency::new("git"),
            WorkstationDependency::new("docker"),
        ]
    });

    let first_uninstalled = use_state(|| String::new());
    let status_checked = use_state(|| false);

    {
        let dependencies = dependencies.clone();
        let first_uninstalled = first_uninstalled.clone();
        let status_checked = status_checked.clone();

        use_effect(move || {
            if !*status_checked {
                spawn_local(async move {
                    let mut deps = (*dependencies).clone();
                    for dep in deps.iter_mut() {
                        dep.installed = match dep.is_installed().await {
                            Ok(installed) => Some(installed),
                            Err(_) => None,
                        };
                    }

                    // Find the first uninstalled dependency and set it as the state
                    if let Some(dep) = deps.iter().find(|dep| dep.installed == Some(false)) {
                        first_uninstalled.set(dep.name.clone());
                    }

                    // Update the dependencies state
                    dependencies.set(deps);
                    status_checked.set(true); // Mark the status check as complete
                });
            }

            || ()
        });
    }

    let toggle = |key: String| {
        let first_uninstalled = first_uninstalled.clone();
        Callback::from(move |_: ()| {
            first_uninstalled.set(key.clone());
        })
    };

    let accordion_items = dependencies.iter().enumerate().map(|(_index, dep)| {
        let title = match dep.installed {
            Some(true) => format!("✅ {}", dep.name),
            Some(false) => format!("⚠️ {}", dep.name),
            None => format!("⏳️ {}", dep.name),
        };

        let on_toggle = toggle(dep.name.clone()); // Pass the name to toggle

        let is_expanded = *first_uninstalled == dep.name; // Check if the current state matches the dependency name

        html_nested! {
            <AccordionItem title={title} expanded={is_expanded} onclick={on_toggle}>
                <div>
            { match dep.installed {
                None => {"Dependency check is pending ...".to_string()},
                Some(b) => format!("{} is {}", dep.name, if b { "installed:" } else { "not installed." })
            }}
//            { format!("{} is {}", dep.name, if dep.installed.unwrap_or(false) { "installed:" } else { "not installed." }) }
            { if dep.installed.unwrap_or(false) && dep.path.len() > 0 {
                html! {
                    <div>
                    <code>
                    { dep.path.to_string() }
                    </code>
                    </div>
                }
            } else { html! {} }}
            </div>
            </AccordionItem>
        }
    }).collect::<Vec<VChild<AccordionItem>>>();

    html! {
        <Accordion>
            { accordion_items }
        </Accordion>
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
