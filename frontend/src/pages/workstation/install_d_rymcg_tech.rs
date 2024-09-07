use crate::components::loading_state::LoadingState;
use crate::components::terminal::{
    EnvVar, EnvVarList, EnvVarProps, TerminalOutput, TerminalOutputProps,
};
use crate::pages::workstation::WorkstationTab;
use dry_console_dto::config::DRymcgTechConfigState;
use dry_console_dto::script::ScriptEntry;
use dry_console_dto::workstation::PathValidationResult;
use gloo::console::{debug, error};
use gloo::net::http::Request;
use gloo::timers::callback::Timeout;
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
    let root_dir_validation = use_state(|| None::<bool>); // Track validation for ROOT_DIR

    // Store the debounce timeout to allow resetting it
    let debounce_timeout = use_mut_ref(|| None::<Timeout>);

    {
        let config_state = config_state.clone();
        let env_vars_state = env_vars_state.clone(); // Clone env_vars_state for use in async block
        let root_dir_validation = root_dir_validation.clone();
        let root_dir_validation2 = root_dir_validation.clone();

        let on_value_change = Callback::from(move |(name, value): (String, String)| {
            // Cancel previous timeout if it exists
            if let Some(timeout) = debounce_timeout.borrow_mut().take() {
                timeout.cancel();
            }

            // Set a new timeout for debouncing (1 second)
            let name_clone = name.clone();
            let value_clone = value.clone();
            let root_dir_validation = root_dir_validation.clone();

            *debounce_timeout.borrow_mut() = Some(Timeout::new(1000, move || {
                let root_dir_validation = root_dir_validation.clone();
                // Fire the callback after 1 second of inactivity
                match name_clone.as_str() {
                    "ROOT_DIR" => {
                        root_dir_validation.set(None);
                        wasm_bindgen_futures::spawn_local(async move {
                            // Perform async HTTP request to check if ROOT_DIR is valid
                            let response = Request::get(&format!(
                                "/api/workstation/filesystem/validate_path/?path={}",
                                value_clone
                            ))
                            .send()
                            .await;
                            if let Ok(response) = response {
                                if response.status() == 200 {
                                    // Deserialize the response
                                    if let Ok(result) =
                                        response.json::<PathValidationResult>().await
                                    {
                                        if result.can_be_created {
                                            root_dir_validation.set(Some(true));
                                        }
                                    } else {
                                        root_dir_validation.set(Some(false));
                                        error!("Failed to deserialize PathValidationResult");
                                    }
                                } else {
                                    root_dir_validation.set(Some(false));
                                    error!("Error: Received non-200 status code");
                                }
                            } else {
                                root_dir_validation.set(Some(false));
                                error!("Error: Failed to send request");
                            }
                        });
                    }
                    _ => {}
                }
            }));
        });

        let root_dir_validation = root_dir_validation2.clone();
        use_effect_with(root_dir_validation.clone(), move |root_dir_validation| {
            let config = config_state.clone();
            let env_vars = env_vars_state.clone();
            let on_value_change = on_value_change.clone(); // Clone callback to use within async

            // Clone or extract the value of root_dir_validation for use in the async block
            let root_dir_validation_value = (*root_dir_validation).clone();

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
                                name: env_var.clone().name,
                                description: env_var.clone().description,
                                help: env_var.clone().help.unwrap_or(Vec::<String>::new()),
                                default_value: env_var.clone().default_value,
                                on_value_change: Some(on_value_change.clone()),
                                // Validation:
                                is_valid: match env_var.clone().name.as_str() {
                                    "ROOT_DIR" => root_dir_validation_value.unwrap_or(false),
                                    _ => false,
                                },
                                ..Default::default()
                            })
                            .collect();

                        env_vars.set(Some(env_var_props));
                    }
                }
            });
            || ()
        });
    }

    let is_valid = (*root_dir_validation).unwrap_or(false);

    if let Some(config) = (*config_state).clone() {
        if let Some(root_dir) = &config.config.root_dir {
            html! { <div>{format!("Already installed at {}.", root_dir)}</div> }
        } else {
            if let Some(env_vars) = (*env_vars_state).clone() {
                html! {
                    <Card>
                        <CardBody>
                        <TerminalOutput script="InstallDRymcgTech" {is_valid} reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}>
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
