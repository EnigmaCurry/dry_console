use crate::api::{APIModule, ApiModule};
use crate::app_state::SharedState;
use crate::API_PREFIX;
use axum::response::Redirect;
use axum::routing::{any, MethodRouter};
use axum::Router;

pub fn route(
    module: APIModule,
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
    let mut router = Router::new().route(&p, method_router);
    if p != "/" {
        // Redirect all URLs missing the final forward-slash /
        let r = Redirect::permanent(format!("{API_PREFIX}/{}{}", module.to_string(), &p).as_str());
        router = router.route(&p.trim_end_matches("/"), any(|| async { r }));
    }
    router
}
