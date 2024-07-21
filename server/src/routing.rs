use std::convert::Infallible;

use crate::api::APIModule;
use crate::app_state::SharedState;

use axum::routing::MethodRouter;
use axum::Router;
use tracing::debug;

pub fn route(
    _module: APIModule,
    path: &str,
    method_router: MethodRouter<SharedState, Infallible>,
) -> Router<SharedState> {
    let p: String = match path.trim_matches('/') {
        "" => "/".to_string(),
        p2 => format!("/{}/", p2),
    };
    debug!("{:?}", p);
    Router::new().route(&p, method_router)
    //     .route(
    //     &p.clone().trim_end_matches("/"),
    //     get(move || async move { Redirect::permanent(&p) }),
    // )
}
