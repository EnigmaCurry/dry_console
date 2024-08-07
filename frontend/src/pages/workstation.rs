use patternfly_yew::prelude::*;
use yew::prelude::*;

mod dependencies;

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
