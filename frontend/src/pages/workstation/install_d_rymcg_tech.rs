use crate::components::loading_state::LoadingState;
use crate::components::terminal::{EnvVar, TerminalOutput, TerminalOutputProps};
use crate::pages::workstation::WorkstationTab;
use dry_console_dto::config::DRymcgTechConfigState;
use gloo::console::debug;
use gloo::net::http::Request;
use patternfly_yew::prelude::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InstallDRyMcGTechProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    let config_state = use_state(|| None::<DRymcgTechConfigState>);
    {
        let config_state = config_state.clone();
        use_effect_with((), move |_| {
            let config = config_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let response = Request::get("/api/workstation/d.rymcg.tech/").send().await;

                if let Ok(response) = response {
                    if let Ok(fetched_config) = response.json::<DRymcgTechConfigState>().await {
                        config.set(Some(fetched_config));
                    }
                }
            });
            || ()
        });
    }

    if let Some(config) = (*config_state).clone() {
        if let Some(root_dir) = &config.config.root_dir {
            html! { <div>{format!("Already installed at {}.", root_dir)}</div> }
        } else {
            html! {
                <Card>
                    <CardTitle><h1>{"Install d.rymcg.tech"}</h1></CardTitle>
                    <CardBody>
                    <TerminalOutput script="InstallDRymcgTech" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}>
                    // if let Some(candidate_root_dir) = &config.candidate_root_dir {
                    //     <div>{format!("Existing installation candidate found: {}.", candidate_root_dir)}</div>
                    // }
                    <EnvVar name="ROOT_DIR" description="Enter the filesystem path to clone the d.rymcg.tech git repository to:"/>
                    </TerminalOutput>
                    </CardBody>
                </Card>
            }
        }
    } else {
        html! { <LoadingState/> }
    }
}
