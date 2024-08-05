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
            <section hidden={(*selected) != HostTab::Dependencies}><DependencyList/></section>
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

    async fn is_installed(self: &Self) -> Result<bool, anyhow::Error> {
        // TODO: check API
        Ok(false)
    }
}

#[function_component(DependencyList)]
fn dependency_list() -> Html {
    let dependencies = use_state::<Vec<HostDependency>, _>(|| {
        vec![
            HostDependency::new("git"),
            HostDependency::new("docker"),
            // Add more dependencies here
        ]
    });

    let cloned_dependencies = dependencies.clone();

    use_effect(move || {
        let dependencies = cloned_dependencies.clone();
        spawn_local(async move {
            let mut deps: Vec<HostDependency> = (*dependencies).clone();
            for dep in deps.iter_mut() {
                match dep.is_installed().await {
                    Ok(installed) => dep.installed = Some(installed),
                    Err(_) => dep.installed = None,
                }
            }
            dependencies.set(deps);
        });
        || ()
    });

    let accordion_items = dependencies.iter().enumerate().map(|(index, dep)| {
        let title = match dep.installed {
            Some(true) => format!("✅ {}", dep.name),
            Some(false) => format!("⚠️ {}", dep.name),
            None => dep.name.clone(),
        };

        html_nested! {
            <AccordionItem title={title} expanded={false}>
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
