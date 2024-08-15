use crate::components::ButtonLink;
use crate::pages::workstation::WorkstationTab;
use gloo::net::http::Request;
use patternfly_yew::prelude::*;
use serde::Deserialize;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew::virtual_dom::VChild;

#[derive(Properties, PartialEq)]
pub struct InstallDRyMcGTechProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    html! {
        <>
        {"here"}
        </>
    }
}
