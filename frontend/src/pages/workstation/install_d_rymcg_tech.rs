use crate::components::terminal::{TerminalOutput, TerminalOutputProps};
use crate::pages::workstation::WorkstationTab;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InstallDRyMcGTechProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    html! {
        <Card>
            <CardTitle><h1>{"Install d.rymcg.tech"}</h1></CardTitle>
            <CardBody>
            <TerminalOutput script="InstallDRymcgTech" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}/>
            <TerminalOutput script="InstallDependencies" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()} />
            <TerminalOutput script="TestExampleOne" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}/>
            </CardBody>
        </Card>
    }
}
