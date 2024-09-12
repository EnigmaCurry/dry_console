use crate::components::loading_state::LoadingState;
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

use wasm_bindgen_futures::spawn_local;

#[allow(clippy::single_match)]
#[function_component(InstallDRyMcGTech)]
pub fn install(props: &InstallDRyMcGTechProps) -> Html {
    // State hooks
    let is_first_render = use_state(|| true);
    let script_name = "InstallDRymcgTech";
    let config_state = use_state(|| None::<DRymcgTechConfigState>);
    let env_vars_state = use_state(|| None::<Vec<EnvVarProps>>);
    let root_dir_validation = use_state(|| Some(false));
    let root_dir_validation_help = use_state(|| None::<String>);
    let refresh_state = use_state(|| 0);
    let is_fresh_install = use_state(|| None::<bool>);
    let debounce_timeout = use_mut_ref(|| None::<Timeout>);

    // Helper function to validate root directory
    let validate_root_dir =
        |path: String,
         root_dir_validation: UseStateHandle<Option<bool>>,
         root_dir_validation_help: UseStateHandle<Option<String>>| {
            spawn_local(async move {
                let response = Request::get(&format!(
                    "/api/workstation/filesystem/validate_path/?path={}",
                    path
                ))
                .send()
                .await;
                if let Ok(response) = response {
                    if response.status() == 200 {
                        if let Ok(result) = response.json::<PathValidationResult>().await {
                            if result.can_be_created {
                                root_dir_validation.set(Some(true));
                                root_dir_validation_help.set(None);
                            } else if result.exists {
                                root_dir_validation.set(Some(false));
                                root_dir_validation_help.set(Some("Error: this directory already exists, please choose a new location".to_string()));
                            } else {
                                root_dir_validation.set(Some(false));
                                root_dir_validation_help.set(None);
                            }
                        }
                    }
                }
            });
        };

    // Helper function for debounced value change handling
    let handle_value_change = {
        let root_dir_validation = root_dir_validation.clone();
        let root_dir_validation_help = root_dir_validation_help.clone();
        let debounce_timeout = debounce_timeout.clone();
        let is_first_render = is_first_render.clone();

        Callback::from(move |(name, value): (String, String)| {
            root_dir_validation.set(None);

            if *is_first_render {
                is_first_render.set(false);
                if !value.is_empty() {
                    validate_root_dir(
                        value.clone(),
                        root_dir_validation.clone(),
                        root_dir_validation_help.clone(),
                    );
                }
                return;
            }

            if let Some(timeout) = debounce_timeout.borrow_mut().take() {
                timeout.cancel();
            }

            if name == "ROOT_DIR" {
                let value_clone = value.clone();
                let root_dir_validation = root_dir_validation.clone();
                let root_dir_validation_help = root_dir_validation_help.clone();

                *debounce_timeout.borrow_mut() = Some(Timeout::new(1000, move || {
                    validate_root_dir(value_clone, root_dir_validation, root_dir_validation_help);
                }));
            }
        })
    };

    let fetch_config_and_env_vars = {
        let config_state = config_state.clone();
        let env_vars_state = env_vars_state.clone();
        let root_dir_validation = root_dir_validation.clone();
        let root_dir_validation_help = root_dir_validation_help.clone();
        let handle_value_change = handle_value_change.clone();

        async move {
            let config_response = Request::get("/api/workstation/d.rymcg.tech/").send().await;
            if let Ok(config_response) = config_response {
                if let Ok(fetched_config) = config_response.json::<DRymcgTechConfigState>().await {
                    config_state.set(Some(fetched_config));
                }
            }

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
                            name: env_var.name.clone(),
                            description: env_var.description.clone(),
                            help: env_var.help.unwrap_or_default(),
                            default_value: env_var.default_value.clone(),
                            on_value_change: Some(handle_value_change.clone()),
                            validation_help: if env_var.name == "ROOT_DIR" {
                                root_dir_validation_help.as_deref().map(|s| s.to_string())
                            } else {
                                None
                            },
                            is_valid: if env_var.name == "ROOT_DIR" {
                                Some(root_dir_validation.unwrap_or(false))
                            } else {
                                Some(false)
                            },
                            ..Default::default()
                        })
                        .collect();
                    env_vars_state.set(Some(env_var_props));
                }
            }
        }
    };

    // Use effect to trigger on mount and check if the default value is non-blank
    {
        let handle_value_change = handle_value_change.clone();
        let env_vars_state = env_vars_state.clone();
        let is_first_render = is_first_render.clone();
        use_effect_with(env_vars_state.clone(), move |_| {
            if let Some(env_vars) = (*env_vars_state).clone() {
                if *is_first_render {
                    for env_var in env_vars.iter() {
                        if !env_var.default_value.is_empty() {
                            handle_value_change
                                .emit((env_var.name.clone(), env_var.default_value.clone()));
                        }
                    }
                }
            }
            || ()
        });
    }

    use_effect_with(
        (
            root_dir_validation.clone(),
            root_dir_validation_help.clone(),
            *refresh_state,
        ),
        move |_| {
            spawn_local(fetch_config_and_env_vars);
            || ()
        },
    );

    let refresh = {
        let refresh_state = refresh_state.clone();
        Callback::from(move |_| {
            refresh_state.set(*refresh_state + 1);
        })
    };

    let on_choose_install = {
        let is_fresh_install = is_fresh_install.clone();
        let root_dir_validation = root_dir_validation.clone();

        Callback::from(move |new_state: Option<bool>| {
            is_fresh_install.set(new_state);
            root_dir_validation.set(Some(false));
        })
    };

    if let Some(config) = (*config_state).clone() {
        if let Some(root_dir) = &config.config.root_dir {
            html! {
                <Card>
                    <CardTitle><h3>{"d.rymcg.tech is now installed:"}</h3></CardTitle>
                    <CardBody>
                        <ul><li><code>{format!("{root_dir}")}</code></li></ul>
                        <Uninstall on_refresh={refresh.clone()}/>
                    </CardBody>
                </Card>
            }
        } else if let Some(candidate_dir) = config.candidate_root_dir {
            html! { <ConfirmInstall root_dir={candidate_dir} on_refresh={refresh.clone()}/> }
        } else {
            match *is_fresh_install {
                None => html! { <ChooseInstall on_choose={on_choose_install}/> },
                Some(true) => {
                    if let Some(env_vars) = (*env_vars_state).clone() {
                        html! {
                            <Card>
                                <CardBody>
                                    <TerminalOutput script="InstallDRymcgTech" is_valid={*root_dir_validation} reload_trigger={props.reload_trigger} selected_tab={props.selected_tab.clone()} on_done={TerminalOutputProps::default_on_done()}>
                                        <ResetInstallChoice on_reset={on_choose_install}/>
                                        <br/>
                                        <EnvVarList env_vars={env_vars}/>
                                    </TerminalOutput>
                                </CardBody>
                            </Card>
                        }
                    } else {
                        html! { <LoadingState/> }
                    }
                }
                Some(false) => html! { <ResetInstallChoice on_reset={on_choose_install}/> },
            }
        }
    } else {
        html! { <LoadingState/> }
    }
}

#[derive(Properties, PartialEq)]
pub struct ResetInstallChoiceProps {
    pub on_reset: Callback<Option<bool>>,
}

#[function_component(ResetInstallChoice)]
pub fn reset_install_choice(props: &ResetInstallChoiceProps) -> Html {
    let on_reset = {
        let on_choose = props.on_reset.clone();
        Callback::from(move |_| {
            on_choose.emit(None);
        })
    };
    html! {
        <div class="button_list">
            <Button onclick={on_reset} class="deny">{"Go back to choose a different installation method"}</Button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ChooseInstallProps {
    pub on_choose: Callback<Option<bool>>,
}

#[function_component(ChooseInstall)]
pub fn choose_install(props: &ChooseInstallProps) -> Html {
    let on_click_fresh_install = {
        let on_choose = props.on_choose.clone();
        Callback::from(move |_| {
            on_choose.emit(Some(true));
        })
    };

    let on_click_use_existing = {
        let on_choose = props.on_choose.clone();
        Callback::from(move |_| {
            on_choose.emit(Some(false));
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
                <p>{"Do you want to import this directory into your config?"}</p>
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

    let deactivate_text = "🧹 Deactivate";
    html! {
        <ExpandableSection toggle_text_expanded={deactivate_text} toggle_text_hidden={deactivate_text}>
        <div class="button_group">
            <Tooltip text={"This will disassociate the active d.rymcg.tech directory from dry_console. No files will be removed. Once removed, you may re-run the wizard to re-install the existing directory, or to a new location."}>
            <Button class="deny" onclick={on_click.clone()}>{"Deactivate d.rymcg.tech"}</Button>
            </Tooltip>
        </div>
        </ExpandableSection>
    }
}
