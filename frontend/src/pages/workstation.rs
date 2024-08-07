use patternfly_yew::prelude::*;
use yew::prelude::*;

mod dependencies;
mod system;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorkstationTab {
    System,
    Dependencies,
}

#[function_component(WorkstationTabs)]
fn tabs() -> Html {
    let selected = use_state_eq(|| WorkstationTab::System);
    let reload_trigger = use_state(|| 0);

    let onselect = {
        let selected = selected.clone();
        let reload_trigger = reload_trigger.clone();
        Callback::from(move |index: WorkstationTab| {
            if index == WorkstationTab::Dependencies {
                reload_trigger.set(*reload_trigger + 1); // Trigger reload when Dependencies tab is selected
            }
            selected.set(index);
        })
    };

    html! (
        <>
            <Tabs<WorkstationTab> detached=true {onselect} selected={*selected} r#box=true>
                <Tab<WorkstationTab> index={WorkstationTab::System} title="System"/>
                <Tab<WorkstationTab> index={WorkstationTab::Dependencies} title="Dependencies"/>
            </Tabs<WorkstationTab>>
            <section hidden={(*selected) != WorkstationTab::System}>
                <system::System />
            </section>
            <section hidden={(*selected) != WorkstationTab::Dependencies} key={*reload_trigger}>
                <dependencies::DependencyList />
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
