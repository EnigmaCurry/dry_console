use std::convert::Infallible;

use crate::api::route as api_route;

use crate::path::{could_create_path, path_is_git_repo_root};
use crate::response::{AppError, AppJson, JsonResult};
use crate::{app_state::SharedState, AppRouter};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Request;
use axum::routing::{post, MethodRouter};
use axum::{extract::State, routing::get, Router};
use dry_console_dto::config::{ConfigData, ConfigSection, DRymcgTechConfigState};
use tracing::debug;

const DEFAULT_D_RYMCG_TECH_ROOT_DIR: &str = "~/git/vendor/enigmacurry/d.rymcg.tech";

pub fn main() -> AppRouter {
    Router::new().merge(config()).merge(confirm_installed())
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
                    let installed = path_is_git_repo_root(section.root_dir.clone());
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

                        if path_is_git_repo_root(default_dir.clone()) {
                            candidate_root_dir = default_dir;
                        } else {
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
                                Some(dir) => match could_create_path(std::path::Path::new(&dir)) {
                                    Ok(_) => Some(dir),
                                    Err(_e) => None,
                                },
                                None => None,
                            };
                        }
                    }
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
    get,
    path = "/api/workstation/d.rymcg.tech/confirm_installed",
    responses(
        (status = OK, description = "Set the existing d.rymcg.tech ROOT_DIR", body = str)
    )
)]
pub fn confirm_installed() -> AppRouter {
    async fn handler(State(state): State<SharedState>, req: Request<Body>) -> JsonResult<bool> {
        let config = {
            let state = state.read().await;
            state.config.clone()
        };
        Ok(AppJson(true))
    }
    route("/confirm_installed", post(handler))
}
