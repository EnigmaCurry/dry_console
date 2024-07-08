use crate::api::APIModule;
use crate::app_state::SharedState;
use axum::routing::MethodRouter;
use axum::Router;

pub fn route(
    _module: APIModule,
    path: &str,
    method_router: MethodRouter<SharedState>,
) -> Router<SharedState> {
    let p: String;
    match path.trim_matches('/') {
        "" => {
            p = "/".to_string();
        }
        p2 => p = format!("/{}/", p2.to_string()),
    }
    let router = Router::new().route(&p, method_router);
    router
}
