use std::convert::Infallible;

use crate::api::route as api_route;

use crate::response::{AppError, AppJson, JsonResult};
use crate::{app_state::SharedState, AppRouter};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Request;
use axum::routing::MethodRouter;
use axum::{extract::State, routing::get, Router};
use dry_console_dto::config::{ConfigSection, DRymcgTechConfig};

pub fn main() -> AppRouter {
    Router::new().merge(config())
}

fn route(path: &str, method_router: MethodRouter<SharedState, Infallible>) -> AppRouter {
    api_route(&format!("/d.rymcg.tech{path}"), method_router)
}

#[utoipa::path(
    get,
    path = "/api/workstation/d.rymcg.tech/config/", 
    responses(
        (status = OK, description = "d.rymcg.tech configuration info", body = str)
    )
)]
pub fn config() -> AppRouter {
    async fn handler(
        State(state): State<SharedState>,
        req: Request<Body>,
    ) -> JsonResult<DRymcgTechConfig> {
        let config = {
            let state = state.read().await;
            state.config.clone()
        };
        match config.sections.get(&ConfigSection::DRymcgTech) {
            Some(cfg) => match serde_json::to_string(&cfg) {
                Ok(s) => match cfg.validate() {
                    Ok(true) => Ok(AppJson(serde_json::from_str(&s)?)),
                    Ok(false) => Err(AppError::Config(
                        anyhow!("Config is invalid."),
                        Some(req.uri().to_string()),
                    )),
                    Err(e) => Err(AppError::Config(
                        anyhow!("Config is invalid: {e}"),
                        Some(req.uri().to_string()),
                    )),
                },
                Err(e) => Err(AppError::Internal(e.into(), Some(req.uri().to_string()))),
            },
            _ => {
                let cfg = DRymcgTechConfig::default();
                match serde_json::to_string(&cfg) {
                    Ok(s) => Ok(AppJson(serde_json::from_str(&s)?)),
                    Err(e) => Err(AppError::Internal(e.into(), Some(req.uri().to_string()))),
                }
            }
        }
    }
    route("/config", get(handler))
}
