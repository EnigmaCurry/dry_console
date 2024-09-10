use crate::components::loading_state::LoadingState;
use crate::components::manual_intervention::_ManualInterventionProps::reload_trigger;
use crate::components::terminal::{EnvVarList, EnvVarProps, TerminalOutput, TerminalOutputProps};
use crate::pages::workstation::WorkstationTab;
use crate::toast::get_toast;
use dry_console_dto::config::DRymcgTechConfigState;
use dry_console_dto::script::ScriptEntry;
use dry_console_dto::workstation::{
    ConfirmInstalledRequest, FreshInstallRequest, UninstallRequest, UseExistingInstallRequest,
};
use dry_console_dto::workstation::{PathValidationResult, PurgeRootDirRequest};
use gloo::console::{debug, error};
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

#[allow(clippy::single_match)]
#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    let script_name = "InstallDRymcgTech";
    let config_state = use_state(|| None::<DRymcgTechConfigState>);
    let env_vars_state = use_state(|| None::<Vec<EnvVarProps>>); // New state for env vars
    let root_dir_validation = use_state(|| Some(false)); // Track validation for ROOT_DIR
    let refresh_state = use_state(|| 0); // Track when this component needs to refresh
    let is_fresh_install = use_state(|| false);

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
        use_effect_with((root_dir_validation.clone(), *refresh_state), move |_| {
            let config = config_state.clone();
            let env_vars = env_vars_state.clone();
            let on_value_change = on_value_change.clone();

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
                                    "ROOT_DIR" => Some(root_dir_validation_value.unwrap_or(false)),
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

    let refresh = {
        let refresh_state = refresh_state.clone();
        Callback::from(move |_| {
            // Increment the refresh state, which will trigger a re-render and re-fetch
            refresh_state.set(*refresh_state + 1);
            //debug!("gonna refresh.");
        })
    };

    if let Some(config) = (*config_state).clone() {
        if let Some(root_dir) = &config.config.root_dir {
            html! {
                <Card>
                    <CardTitle>
                    <h3>{"d.rymcg.tech is now installed:"}</h3>
                    </CardTitle>
                    <CardBody>
                    <ul><li><code>{format!("{root_dir}")}</code></li></ul>
                    <Uninstall on_refresh={refresh.clone()}/>
                    </CardBody>
                </Card>
            }
        } else if let Some(candidate_dir) = config.candidate_root_dir {
            html! {
                <ConfirmInstall root_dir={candidate_dir} on_refresh={refresh.clone()}/>
            }
        } else if *is_fresh_install {
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
        } else {
            html! { <ChooseInstall on_refresh={refresh.clone()} /> }
        }
    } else {
        html! { <LoadingState/> }
    }
}

#[derive(Properties, PartialEq)]
pub struct ChooseInstallProps {
    pub on_refresh: Callback<()>,
}

#[function_component(ChooseInstall)]
pub fn choose_install(props: &ChooseInstallProps) -> Html {
    let on_refresh = props.on_refresh.clone();

    let on_click_use_existing = {
        let on_refresh = on_refresh.clone();
        Callback::from(move |_| {
            let on_refresh = on_refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::to_string(&UseExistingInstallRequest {})
                    .expect("Failed to serialize request.");
                let request_result =
                    Request::post("/api/workstation/d.rymcg.tech/use_existing_install/")
                        .header("Content-Type", "application/json")
                        .body(body);
                if let Ok(request) = request_result {
                    let response = request.send().await;
                    match response {
                        Ok(resp) if resp.ok() => {
                            //log::debug!("API call successful!");
                            on_refresh.emit(());
                        }
                        Ok(resp) => {
                            log::error!("API error: {:?}", resp);
                        }
                        Err(err) => {
                            log::error!("Request failed: {:?}", err);
                        }
                    }
                } else {
                    log::error!("Failed to create request.");
                }
            });
        })
    };

    let on_click_fresh_install = {
        let on_refresh = on_refresh.clone();
        Callback::from(move |_| {
            let on_refresh = on_refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::to_string(&FreshInstallRequest {})
                    .expect("Failed to serialize request.");
                let request_result = Request::post("/api/workstation/d.rymcg.tech/fresh_install/")
                    .header("Content-Type", "application/json")
                    .body(body);
                if let Ok(request) = request_result {
                    let response = request.send().await;
                    match response {
                        Ok(resp) if resp.ok() => {
                            //log::debug!("API call successful!");
                            on_refresh.emit(());
                        }
                        Ok(resp) => {
                            log::error!("API error: {:?}", resp);
                        }
                        Err(err) => {
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
            <h3>{"Install d.rymcg.tech"}</h3>
            </CardTitle>
            <CardBody>
            <p><a href={"https://github.com/EnigmaCurry/d.rymcg.tech"}>{"d.rymcg.tech"}</a>{" is a configuration and deployment environment for Docker (docker-compose), and it is a prerequisite of dry_console."}</p>
            <br/>
            <p>{"Please choose the installation source:"}</p>
            <br/>
            <ul class="button_list">
            <li><Button class="confirm" onclick={on_click_fresh_install} >{"Install d.rymcg.tech in a new directory"}</Button></li>
            <li><Button class="alt" onclick={on_click_use_existing} >{"Import an existing installation directory"}</Button></li>
            </ul>
            </CardBody>
        </Card>
    }
}

#[derive(Properties, PartialEq)]
pub struct ConfirmInstallProps {
    pub root_dir: String,
    pub on_refresh: Callback<()>,
}

#[function_component(ConfirmInstall)]
pub fn confirm_install(props: &ConfirmInstallProps) -> Html {
    let candidate_dir = rc::Rc::new(props.root_dir.clone());
    let on_refresh = props.on_refresh.clone();

    let on_click_deny = {
        let on_refresh = on_refresh.clone();
        Callback::from(move |_| {
            let on_refresh = on_refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::to_string(&PurgeRootDirRequest {})
                    .expect("Failed to serialize request.");
                let request_result = Request::post("/api/workstation/d.rymcg.tech/purge_root_dir/")
                    .header("Content-Type", "application/json")
                    .body(body);
                if let Ok(request) = request_result {
                    let response = request.send().await;
                    match response {
                        Ok(resp) if resp.ok() => {
                            //log::debug!("API call successful!");
                            on_refresh.emit(());
                        }
                        Ok(resp) => {
                            log::error!("API error: {:?}", resp);
                        }
                        Err(err) => {
                            log::error!("Request failed: {:?}", err);
                        }
                    }
                } else {
                    log::error!("Failed to create request.");
                }
            });
        })
    };

    let on_click_confirm = {
        let candidate_dir = candidate_dir.clone();
        Callback::from(move |_| {
            let candidate_dir = candidate_dir.clone();
            let on_refresh = on_refresh.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::to_string(&ConfirmInstalledRequest {
                    root_dir: (*candidate_dir).clone(),
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
                            //log::debug!("API call successful!");
                            on_refresh.emit(());
                        }
                        Ok(resp) => {
                            log::error!("API error: {:?}", resp);
                        }
                        Err(err) => {
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
            <h3>{"Install d.rymcg.tech"}</h3>
            </CardTitle>
            <CardBody>
                <p>{"It looks like d.rymcg.tech may already be installed."}</p>
                <br/>
                <p>{format!("Please examine this directory on your workstation:")}</p>
                <ul><li><code>{&*candidate_dir}</code></li></ul>
                <p>{"Do you you want to import this directory into your config?"}</p>
                <div class="button_group">
                    <Button class="deny" onclick={on_click_deny} >{"No, use a different directory"}</Button>
                    <Button class="confirm" onclick={on_click_confirm} >{"Yes, use this directory"}</Button>
                </div>
            </CardBody>
        </Card>
    }
}

#[derive(Properties, PartialEq)]
pub struct UninstallProps {
    pub on_refresh: Callback<()>,
}

#[function_component(Uninstall)]
pub fn uninstall(props: &UninstallProps) -> Html {
    let on_refresh = props.on_refresh.clone();
    let toast = get_toast(use_toaster().expect("Must be nested inside a ToastViewer"));
    let on_click = {
        let toast = toast.clone();
        Callback::from(move |_| {
            let on_refresh = on_refresh.clone();
            let toast = toast.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let body = serde_json::to_string(&UninstallRequest {})
                    .expect("Failed to serialize request.");
                let request_result = Request::post("/api/workstation/d.rymcg.tech/uninstall/")
                    .header("Content-Type", "application/json")
                    .body(body);
                if let Ok(request) = request_result {
                    let response = request.send().await;
                    match response {
                        Ok(resp) if resp.ok() => {
                            //log::debug!("API call successful!");
                            on_refresh.emit(());
                            toast(AlertType::Success, "d.rymcg.tech uninstalled!");
                        }
                        Ok(resp) => {
                            log::error!("API error: {:?}", resp);
                        }
                        Err(err) => {
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
        <ExpandableSection toggle_text_expanded={"🧹 Uninstall"} toggle_text_hidden={"🧹 Deactivate"}>
        <div class="button_group">
            <Tooltip text={"This will disassociate the active d.rymcg.tech directory from dry_console. No files will be removed. Once removed, you may re-run the wizard to re-install the existing directory, or to a new location."}>
            <Button class="deny" onclick={on_click.clone()}>{"Deactivate d.rymcg.tech"}</Button>
            </Tooltip>
        </div>
        </ExpandableSection>
    }
}
