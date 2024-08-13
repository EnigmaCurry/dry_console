use gloo_events::EventListener;
use gloo_utils::window;
use patternfly_yew::prelude::*;
use std::str::FromStr;
use strum_macros::{AsRefStr, EnumString};
use yew::prelude::*;

mod dependencies;
mod system;

#[derive(Clone, PartialEq, Eq, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
enum WorkstationTab {
    System,
    Dependencies,
    DRymcgTech,
}

#[function_component(WorkstationTabs)]
fn tabs() -> Html {
    let selected = use_state_eq(|| {
        let location = window().location();
        let hash = location.hash().unwrap_or_default();
        WorkstationTab::from_str(&hash.trim_start_matches('#')).unwrap_or(WorkstationTab::System)
    });

    let reload_trigger = use_state(|| 0);

    {
        let selected = selected.clone();
        let reload_trigger = reload_trigger.clone();
        use_effect(move || {
            let window = window();
            let listener = EventListener::new(&window.clone(), "hashchange", move |_event| {
                let location = window.location();
                let hash = location.hash().unwrap_or_default();
                if let Ok(tab) = WorkstationTab::from_str(&hash.trim_start_matches('#')) {
                    if tab == WorkstationTab::Dependencies {
                        reload_trigger.set(*reload_trigger + 1); // Trigger reload when Dependencies tab is selected via hash change
                    }
                    selected.set(tab);
                }
            });
            listener.forget(); // Forget the listener to keep it active
            || ()
        });
    }

    let onselect = {
        let selected = selected.clone();
        let reload_trigger = reload_trigger.clone();
        Callback::from(move |index: WorkstationTab| {
            window().location().set_hash(index.as_ref()).unwrap();
            if index == WorkstationTab::Dependencies {
                reload_trigger.set(*reload_trigger + 1);
            }
            selected.set(index);
        })
    };

    html! (
        <>
            <Tabs<WorkstationTab> detached=true {onselect} selected={(*selected).clone()} r#box=true>
                <Tab<WorkstationTab> index={WorkstationTab::System} title="System"/>
                <Tab<WorkstationTab> index={WorkstationTab::Dependencies} title="Dependencies"/>
                <Tab<WorkstationTab> index={WorkstationTab::DRymcgTech} title="d.rymcg.tech"/>
            </Tabs<WorkstationTab>>
            <section hidden={(*selected) != WorkstationTab::System}>
                <system::System />
            </section>
            <section hidden={(*selected) != WorkstationTab::Dependencies} key={*reload_trigger}>
                <dependencies::DependencyList reload_trigger={*reload_trigger} selected_tab={(*selected).clone()} />
            </section>
        </>
    )
}

#[function_component(Workstation)]
pub fn workstation() -> Html {
    html! {
        <PageSection>
            <WorkstationTabs/>
        </PageSection>
    }
}
