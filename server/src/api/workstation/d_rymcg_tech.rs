use std::convert::Infallible;

use crate::api::route as api_route;

use crate::response::{AppError, AppJson, JsonResult};
use crate::{app_state::SharedState, AppRouter};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Request;
use axum::routing::MethodRouter;
use axum::{extract::State, routing::get, Router};
use dry_console_dto::config::{ConfigData, ConfigSection, DRymcgTechConfig};
use tracing::debug;

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
            Some(cfg) => match cfg {
                ConfigData::DRymcgTech(d_rymcg_tech_config) => {
                    match d_rymcg_tech_config.validate() {
                        Ok(true) => {
                            debug!("cfg: {:?}", d_rymcg_tech_config);
                            Ok(AppJson(d_rymcg_tech_config.clone()))
                        }
                        Ok(false) => Err(AppError::Config(
                            anyhow!("Config is invalid."),
                            Some(req.uri().to_string()),
                        )),
                        Err(e) => Err(AppError::Config(
                            anyhow!("Config is invalid: {e}"),
                            Some(req.uri().to_string()),
                        )),
                    }
                }
            },
            None => Err(AppError::Config(
                anyhow!("Missing d.rymcg.tech config section."),
                Some(req.uri().to_string()),
            )),
        }
    }
    route("/config", get(handler))
}
