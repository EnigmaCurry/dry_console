use crate::pages::workstation::WorkstationTab;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InstallDRyMcGTechProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(InstallDRyMcGTech)]
pub fn install(_props: &InstallDRyMcGTechProps) -> Html {
    html! {
        <>
        {"here"}
        </>
    }
}
