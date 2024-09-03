use crate::components::loading_state::LoadingState;
use crate::components::terminal::{EnvVar, TerminalOutput, TerminalOutputProps};
use crate::pages::workstation::WorkstationTab;
use dry_console_dto::config::{DRymcgTechConfig, DRymcgTechConfigState};
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
    let config = use_state(|| None::<DRymcgTechConfigState>);
    {
        let config = config.clone();
        use_effect_with((), move |_| {
            let config = config.clone();
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

    if let Some(config) = (*config).clone() {
        if let Some(_root_dir) = &config.config.root_dir {
            if config.installed {
                html! { <div>{"Already installed."}</div> }
            } else {
                html! {
                    <Card>
                        <CardTitle><h1>{"Install d.rymcg.tech"}</h1></CardTitle>
                        <CardBody>
                        <TerminalOutput script="InstallDRymcgTech" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}>
                        <EnvVar name="ROOT_DIR" description="The path to clone the d.rymcg.tech git repository to."/>
                        </TerminalOutput>
                        </CardBody>
                    </Card>
                }
            }
        } else {
            html! {
                <Card>
                    <CardTitle><h1>{"Install d.rymcg.tech"}</h1></CardTitle>
                    <CardBody>
                    <TerminalOutput script="InstallDRymcgTech" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}>
                    <EnvVar name="ROOT_DIR" description="The path to clone the d.rymcg.tech git repository to."/>
                    </TerminalOutput>
                    </CardBody>
                </Card>
            }
        }
    } else {
        html! { <LoadingState/> }
    }
}
