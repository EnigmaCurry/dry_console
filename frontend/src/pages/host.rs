use anyhow::anyhow;
use patternfly_yew::prelude::*;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VChild;

#[derive(Clone, Copy, PartialEq, Eq)]
enum HostTab {
    Dependencies,
}

#[function_component(HostTabs)]
fn tabs() -> Html {
    let selected = use_state_eq(|| HostTab::Dependencies);
    let onselect = use_callback(selected.clone(), |index, selected| selected.set(index));

    html! (
        <>
            <Tabs<HostTab> detached=true {onselect} selected={*selected} r#box=true>
                <Tab<HostTab> index={HostTab::Dependencies} title="Dependencies"/>
            </Tabs<HostTab>>
            <section hidden={(*selected) != HostTab::Dependencies}>
                <DependencyList/>
            </section>
        </>
    )
}

#[derive(Clone)]
struct HostDependency {
    name: String,
    installed: Option<bool>,
}

impl HostDependency {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            installed: None,
        }
    }

    async fn is_installed(&self) -> Result<bool, anyhow::Error> {
        // TODO: Replace with actual API call to check if installed
        //Err(anyhow!("todo"))
        Ok(false)
    }
}

#[function_component(DependencyList)]
fn dependency_list() -> Html {
    let dependencies = use_state(|| {
        vec![
            HostDependency::new("git"),
            HostDependency::new("docker"),
            // Add more dependencies here
        ]
    });
    let is_checked = use_state(|| false);

    {
        let dependencies = dependencies.clone();
        let is_checked = is_checked.clone();

        use_effect(move || {
            if *is_checked {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }

            spawn_local(async move {
                let mut deps = (*dependencies).clone();
                for dep in deps.iter_mut() {
                    match dep.is_installed().await {
                        Ok(installed) => dep.installed = Some(installed),
                        Err(_) => dep.installed = None,
                    }
                }
                dependencies.set(deps);
                is_checked.set(true); // Mark dependencies as checked
            });

            Box::new(|| ()) as Box<dyn FnOnce()>
        });
    }

    let state = use_state(|| String::new());
    let toggle = |key: String| {
        let state = state.clone();
        Callback::from(move |_: ()| {
            state.set(key.clone()); // Clone the key inside the closure
        })
    };

    let accordion_items = dependencies.iter().enumerate().map(|(_index, dep)| {
        let title = match dep.installed {
            Some(true) => format!("✅ {}", dep.name),
            Some(false) => format!("⚠️ {}", dep.name),
            None => dep.name.clone(),
        };

        let on_toggle = toggle(dep.name.clone());
        let is_expanded = *state == dep.name;
        html_nested! {
            <AccordionItem title={title} expanded={is_expanded} onclick={on_toggle}>
                <div>
            { format!("{} is {}", dep.name, if dep.installed.unwrap_or(false) { "installed" } else { "not installed" }) }
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

#[function_component(Host)]
pub fn host() -> Html {
    html! {
        <PageSection>
            <HostTabs/>
        </PageSection>
    }
}
