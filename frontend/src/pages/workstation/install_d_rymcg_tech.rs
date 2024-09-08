use crate::components::loading_state::LoadingState;
use crate::components::terminal::{
    EnvVarList, EnvVarProps, TerminalOutput, TerminalOutputProps,
};
use crate::pages::workstation::WorkstationTab;
use dry_console_dto::config::DRymcgTechConfigState;
use dry_console_dto::script::ScriptEntry;
use dry_console_dto::workstation::ConfirmInstalledRequest;
use dry_console_dto::workstation::PathValidationResult;
use gloo::console::{error};
use gloo::net::http::Request;
use gloo::timers::callback::Timeout;
use patternfly_yew::prelude::*;
use std::rc;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InstallDRyMcGTechProps {
    pub reload_trigger: u32,
    pub selected_tab: WorkstationTab,
}

#[derive(Properties, PartialEq)]
pub struct ConfirmInstallProps {
    pub root_dir: String,
}

#[function_component(ConfirmInstall)]
pub fn confirm_install(props: &ConfirmInstallProps) -> Html {
    let candidate_dir = rc::Rc::new(props.root_dir.clone());

    let on_click = {
        let candidate_dir = candidate_dir.clone(); // Clone the Rc here, not the string itself
        Callback::from(move |_| {
            let candidate_dir = candidate_dir.clone(); // Clone the Rc again inside the closure
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::to_string(&ConfirmInstalledRequest {
                    root_dir: (*candidate_dir).clone(), // Dereference Rc to access the inner String
                })
                .expect("Failed to serialize request.");

                let request_result =
                    Request::post("/api/workstation/d.rymcg.tech/confirm_installed/")
                        .header("Content-Type", "application/json")
                        .body(body);

                if let Ok(request) = request_result {
                    let response = request.send().await;

                    match response {
                        Ok(resp) if resp.ok() => {
                            // Handle success (e.g., show a success message or redirect)
                            //log::debug!("API call successful!");
                        }
                        Ok(resp) => {
                            // Handle API errors
                            log::error!("API error: {:?}", resp);
                        }
                        Err(err) => {
                            // Handle network or other errors
                            log::error!("Request failed: {:?}", err);
                        }
                    }
                } else {
                    log::error!("Failed to create request.");
                }
            });
        })
    };

    html! {
        <Card>
            <CardTitle>
                <h3>{"It looks like d.rymcg.tech may already be installed"}</h3>
            </CardTitle>
            <CardBody>
                <p>{format!("Please examine this directory:")}</p>
                <ul><li><code>{&*candidate_dir}</code></li></ul>
                <p>{"Does this directory contain an existing installation that you wish to import?"}</p>
                <div class="button_group">
                    <Button class="deny" onclick={on_click.clone()} >{"No, use a different directory"}</Button>
                    <Button class="confirm" onclick={on_click} >{"Yes, use this directory"}</Button>
                </div>
            </CardBody>
        </Card>
    }
}

#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    let script_name = "InstallDRymcgTech";
    let config_state = use_state(|| None::<DRymcgTechConfigState>);
    let env_vars_state = use_state(|| None::<Vec<EnvVarProps>>); // New state for env vars
    let root_dir_validation = use_state(|| Some(false)); // Track validation for ROOT_DIR

    // Store the debounce timeout to allow resetting it
    let debounce_timeout = use_mut_ref(|| None::<Timeout>);

    {
        let config_state = config_state.clone();
        let env_vars_state = env_vars_state.clone(); // Clone env_vars_state for use in async block
        let root_dir_validation = root_dir_validation.clone();
        let root_dir_validation2 = root_dir_validation.clone();

        let on_value_change = Callback::from(move |(name, value): (String, String)| {
            // Reset validation:
            root_dir_validation.set(None);
            let root_dir_validation = root_dir_validation.clone();

            // Cancel previous timeout if it exists
            if let Some(timeout) = debounce_timeout.borrow_mut().take() {
                timeout.cancel();
            }

            // Set a new timeout for debouncing (1 second)
            let name_clone = name.clone();
            let value_clone = value.clone();

            *debounce_timeout.borrow_mut() = Some(Timeout::new(1000, move || {
                let root_dir_validation = root_dir_validation.clone();
                // Fire the callback after 1 second of inactivity
                match name_clone.as_str() {
                    "ROOT_DIR" => {
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
                                        } else {
                                            root_dir_validation.set(Some(false));
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
                                help: env_var.clone().help.unwrap_or_default(),
                                default_value: env_var.clone().default_value,
                                on_value_change: Some(on_value_change.clone()),
                                // Validation:
                                is_valid: match env_var.clone().name.as_str() {
                                    "ROOT_DIR" => *root_dir_validation_value,
                                    _ => Some(false),
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

    let is_valid = *root_dir_validation;

    if let Some(config) = (*config_state).clone() {
        if let Some(root_dir) = &config.config.root_dir {
            html! {
                <Card>
                    <CardTitle>
                    <h3>{"d.rymcg.tech is already installed"}</h3>
                    </CardTitle>
                    <CardBody>
                    <p>{root_dir}</p>
                    </CardBody>
                </Card>
            }
        } else if let Some(candidate_dir) = config.candidate_root_dir {
            html! {
                <ConfirmInstall root_dir={candidate_dir}/>
            }
        } else if let Some(env_vars) = (*env_vars_state).clone() {
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
    } else {
        html! { <LoadingState/> }
    }
}
