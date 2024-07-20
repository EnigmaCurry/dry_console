use crate::api::APIModule;
use crate::app_state::SharedState;
use axum::routing::MethodRouter;
use axum::Router;

pub fn route(
    _module: APIModule,
    path: &str,
    method_router: MethodRouter<SharedState>,
) -> Router<SharedState> {
    let p: String = match path.trim_matches('/') {
        "" => "/".to_string(),
        p2 => format!("/{}/", p2),
    };

    Router::new().route(&p, method_router)
}
