use crate::components::terminal::TerminalOutput;
use crate::pages::workstation::WorkstationTab;

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InstallDRyMcGTechProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    html! {
        <>
            <TerminalOutput reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()}/>
        </>
    }
}
