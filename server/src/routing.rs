use crate::api::APIModule;
use crate::app_state::SharedState;
use crate::AppMethodRouter;
use axum::Router;

pub fn route(
    _module: APIModule,
    path: &str,
    method_router: AppMethodRouter,
) -> Router<SharedState> {
    let p: String = match path.trim_matches('/') {
        "" => "/".to_string(),
        p2 => format!("/{}/", p2),
    };
    if p == "/" {
        Router::new().route(&p, method_router)
    } else {
        Router::new().route(&p, method_router)
        // Router::new().route(&p, method_router.clone()).route(
        //     &p.clone().trim_end_matches("/"),
        //     get(move || async move { Redirect::permanent(&p) }),
        // )
    }
}
