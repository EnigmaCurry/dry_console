use crate::components::loading_state::LoadingState;
use crate::components::terminal::{
    EnvVar, EnvVarList, EnvVarProps, TerminalOutput, TerminalOutputProps,
};
use crate::pages::workstation::WorkstationTab;
use dry_console_dto::config::DRymcgTechConfigState;
use dry_console_dto::script::ScriptEntry;
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
    let script_name = "InstallDRymcgTech";
    let config_state = use_state(|| None::<DRymcgTechConfigState>);
    let env_vars_state = use_state(|| None::<Vec<EnvVarProps>>); // New state for env vars

    {
        let config_state = config_state.clone();
        let env_vars_state = env_vars_state.clone(); // Clone env_vars_state for use in async block

        use_effect_with((), move |_| {
            let config = config_state.clone();
            let env_vars = env_vars_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Fetch config state
                let config_response = Request::get("/api/workstation/d.rymcg.tech/").send().await;
                if let Ok(config_response) = config_response {
                    if let Ok(fetched_config) =
                        config_response.json::<DRymcgTechConfigState>().await
                    {
                        config.set(Some(fetched_config));
                    }
                }

                // Fetch ScriptEntry for environment variables
                let script_response =
                    Request::get(&format!("/api/workstation/command/{}/", script_name))
                        .send()
                        .await;

                if let Ok(script_response) = script_response {
                    if let Ok(script_entry) = script_response.json::<ScriptEntry>().await {
                        let env_var_props: Vec<EnvVarProps> = script_entry
                            .env
                            .into_iter()
                            .map(|env_var| EnvVarProps {
                                name: env_var.name,
                                description: env_var.description,
                                default_value: env_var.default_value,
                                ..EnvVarProps::default()
                            })
                            .collect();

                        env_vars.set(Some(env_var_props));
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
            if let Some(env_vars) = (*env_vars_state).clone() {
                html! {
                    <Card>
                        <CardTitle><h1>{"Install d.rymcg.tech"}</h1></CardTitle>
                        <CardBody>
                            <TerminalOutput script="InstallDRymcgTech" reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}>
                            <EnvVarList env_vars={env_vars}/>
                            </TerminalOutput>
                        </CardBody>
                    </Card>
                }
            } else {
                html! { <LoadingState/> }
            }
        }
    } else {
        html! { <LoadingState/> }
    }
}
