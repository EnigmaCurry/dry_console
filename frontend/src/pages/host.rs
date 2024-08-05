use patternfly_yew::prelude::*;
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

    async fn get_title(self: &Self) -> String {
        match self.is_installed().await {
            Ok(i) => match i {
                true => format!("✅ {}", self.name),
                false => format!("⚠️ {}", self.name),
            },
            Err(_e) => self.name.to_string(),
        }
    }
}

#[function_component(DependencyList)]
fn dependency_list() -> Html {
    let dependencies = vec![
        HostDependency::new("git"),
        HostDependency::new("docker"),
        // Add more dependencies here
    ];
    let state = use_state(|| "");
    let toggle = |key: &'static str| {
        let state = state.clone();
        Callback::from(move |_: ()| {
            state.set(key);
        })
    };

    let first_uninstalled = dependencies.iter().position(|dep| !dep.installed);

    let accordion_items = dependencies.iter().enumerate().map(|(index, dep)| {
        html_nested! {
            <AccordionItem title={dep.get_title()} expanded={first_uninstalled == Some(index)}>
                <div>
                    { format!("{} is {}", dep.name, if dep.installed { "installed" } else { "not installed" }) }
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
