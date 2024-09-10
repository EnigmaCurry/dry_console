use std::convert::Infallible;

use crate::api::route as api_route;

use crate::config::save_config;
use crate::path::{could_create_path, path_is_git_repo_root};
use crate::response::{AppError, AppJson, JsonResult};
use crate::{app_state::SharedState, AppRouter};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Request;
use axum::routing::{post, MethodRouter};
use axum::Json;
use axum::{extract::State, routing::get, Router};
use dry_console_dto::config::{ConfigData, ConfigSection, DRymcgTechConfig, DRymcgTechConfigState};
use dry_console_dto::workstation::ConfirmInstalledRequest;
use tracing::debug;

const DEFAULT_D_RYMCG_TECH_ROOT_DIR: &str = "~/git/vendor/enigmacurry/d.rymcg.tech";

pub fn main() -> AppRouter {
    Router::new()
        .merge(config())
        .merge(confirm_installed())
        .merge(uninstall())
        .merge(purge_root_dir())
}

fn route(path: &str, method_router: MethodRouter<SharedState, Infallible>) -> AppRouter {
    api_route(&format!("/d.rymcg.tech{path}"), method_router)
}

#[utoipa::path(
    get,
    path = "/api/workstation/d.rymcg.tech/", 
    responses(
        (status = OK, description = "d.rymcg.tech configuration info", body = str)
    )
)]
pub fn config() -> AppRouter {
    async fn handler(
        State(state): State<SharedState>,
        req: Request<Body>,
    ) -> JsonResult<DRymcgTechConfigState> {
        let config = {
            let state = state.read().await;
            state.config.clone()
        };
        match config.sections.get(&ConfigSection::DRymcgTech) {
            Some(cfg) => match cfg {
                ConfigData::DRymcgTech(section) => {
                    let installed = section.root_dir.clone().is_some();
                    let mut suggested_root_dir = None;
                    let mut candidate_root_dir = None;
                    if !installed {
                        let default_dir =
                            if DEFAULT_D_RYMCG_TECH_ROOT_DIR.to_string().starts_with('~') {
                                dirs::home_dir().map(|home| {
                                    DEFAULT_D_RYMCG_TECH_ROOT_DIR.replacen(
                                        '~',
                                        &home.to_string_lossy(),
                                        1,
                                    )
                                })
                            } else {
                                Some(DEFAULT_D_RYMCG_TECH_ROOT_DIR.to_string())
                            };
                        match section.previous_root_dir.clone() {
                            // There might exist a previous install dir we'll try:
                            Some(d) => {
                                if path_is_git_repo_root(Some(d.clone())) {
                                    candidate_root_dir = Some(d.clone());
                                } else {
                                    candidate_root_dir = None
                                }
                            }
                            // There is no previous_root_dir, so use the default suggestion:
                            None => {
                                // Expand the "~" in the suggested path
                                suggested_root_dir =
                                    if DEFAULT_D_RYMCG_TECH_ROOT_DIR.to_string().starts_with('~') {
                                        if let Some(home) = dirs::home_dir() {
                                            Some(DEFAULT_D_RYMCG_TECH_ROOT_DIR.replacen(
                                                '~',
                                                &home.to_string_lossy(),
                                                1,
                                            ))
                                        } else {
                                            default_dir.clone()
                                        }
                                    } else {
                                        default_dir.clone()
                                    };
                                // Debugging: Ensure the suggested path is set correctly
                                if let Some(ref dir) = suggested_root_dir {
                                    debug!("Checking path: {}", dir);
                                }
                                // Check that the suggested directory could actually be created
                                suggested_root_dir = match suggested_root_dir {
                                    Some(dir) => {
                                        match could_create_path(std::path::Path::new(&dir)) {
                                            Ok(_) => Some(dir),
                                            Err(_e) => None,
                                        }
                                    }
                                    None => None,
                                };
                            }
                        }
                    };
                    Ok(AppJson(DRymcgTechConfigState {
                        config: section.clone(),
                        installed,
                        suggested_root_dir,
                        candidate_root_dir,
                    }))
                }
            },
            None => Err(AppError::Config(
                anyhow!("Missing d.rymcg.tech config section."),
                Some(req.uri().to_string()),
            )),
        }
    }
    route("/", get(handler))
}

#[utoipa::path(
    post,
    path = "/api/workstation/d.rymcg.tech/confirm_installed",
    request_body = ConfirmInstalledRequest,
    responses(
        (status = OK, description = "Set the existing d.rymcg.tech ROOT_DIR", body = bool)
    )
)]
pub fn confirm_installed() -> AppRouter {
    async fn handler(
        State(state): State<SharedState>,
        Json(request_body): Json<ConfirmInstalledRequest>,
    ) -> JsonResult<bool> {
        let root_dir = request_body.root_dir.clone();

        let mut state = state.write().await;
        let mut config = state.config.clone();

        if let Some(ConfigData::DRymcgTech(ref mut d_rymcg_tech_config)) =
            // Update ROOT_DIR config:
            config.sections.get_mut(&ConfigSection::DRymcgTech)
        {
            d_rymcg_tech_config.root_dir = Some(root_dir.clone());
        } else {
            return Err(AppError::Internal(
                anyhow!("Config section 'd.rymcg.tech' not found"),
                None,
            ));
        }

        // Persist the updated config back into shared state:
        state.config = config.clone();
        // Write config to disk:
        _ = save_config(&config, &state.opt.config_path);
        Ok(AppJson(true)) // Return success
    }

    route("/confirm_installed", post(handler))
}

#[utoipa::path(
    post,
    path = "/api/workstation/d.rymcg.tech/uninstall",
    request_body = UninstallRequest,
    responses(
        (status = OK, description = "Unset the d.rymcg.tech ROOT_DIR", body = bool)
    )
)]
pub fn uninstall() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<bool> {
        let mut state = state.write().await;
        let mut config = state.config.clone();

        if let Some(ConfigData::DRymcgTech(ref mut d_rymcg_tech_config)) =
            // Update ROOT_DIR config:
            config.sections.get_mut(&ConfigSection::DRymcgTech)
        {
            d_rymcg_tech_config
                .previous_root_dir
                .clone_from(&d_rymcg_tech_config.root_dir);
            d_rymcg_tech_config.root_dir = None;
        } else {
            return Err(AppError::Internal(
                anyhow!("Config section 'd.rymcg.tech' not found"),
                None,
            ));
        }

        // Persist the updated config back into shared state:
        state.config = config.clone();
        // Write config to disk:
        _ = save_config(&config, &state.opt.config_path);
        Ok(AppJson(true))
    }

    route("/uninstall", post(handler))
}

#[utoipa::path(
    post,
    path = "/api/workstation/d.rymcg.tech/purge_root_dir",
    request_body = UninstallRequest,
    responses(
        (status = OK, description = "Unset the active ROOT_DIR and the previous uninstalled ROOT_DIR", body = bool)
    )
)]
pub fn purge_root_dir() -> AppRouter {
    async fn handler(State(state): State<SharedState>) -> JsonResult<bool> {
        let mut state = state.write().await;
        let mut config = state.config.clone();

        if let Some(ConfigData::DRymcgTech(ref mut d_rymcg_tech_config)) =
            // Update ROOT_DIR config:
            config.sections.get_mut(&ConfigSection::DRymcgTech)
        {
            d_rymcg_tech_config.previous_root_dir = None;
            d_rymcg_tech_config.root_dir = None;
        } else {
            return Err(AppError::Internal(
                anyhow!("Config section 'd.rymcg.tech' not found"),
                None,
            ));
        }

        // Persist the updated config back into shared state:
        state.config = config.clone();
        // Write config to disk:
        _ = save_config(&config, &state.opt.config_path);
        Ok(AppJson(true))
    }
    route("/purge_root_dir", post(handler))
}

pub fn default_config() -> DRymcgTechConfig {
    let default_dir = if DEFAULT_D_RYMCG_TECH_ROOT_DIR.to_string().starts_with('~') {
        dirs::home_dir()
            .map(|home| DEFAULT_D_RYMCG_TECH_ROOT_DIR.replacen('~', &home.to_string_lossy(), 1))
    } else {
        Some(DEFAULT_D_RYMCG_TECH_ROOT_DIR.to_string())
    };
    let mut d_rymcg_tech_config = DRymcgTechConfig::default();
    if path_is_git_repo_root(default_dir.clone()) {
        d_rymcg_tech_config.previous_root_dir = default_dir;
    }
    d_rymcg_tech_config
}
