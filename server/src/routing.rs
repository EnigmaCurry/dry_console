use crate::api::APIModule;
use crate::app_state::SharedState;
use axum::handler::Handler;
use axum::http::Method;
use axum::response::Redirect;
use axum::routing::{any, get, post, MethodRouter};
use axum::Router;

struct Route<S> {
    method: Method,
    path: &'static str,
    handler: MethodRouter<S>,
}

impl<S> Route<S> {
    fn new<H, T: 'static>(path: &'static str, method: Method, handler: H) -> Self
    where
        H: Handler<T, S> + Clone + Send + 'static,
        H::Future: Send,
        S: Send + Sync + Clone + 'static,
    {
        let handler = match method {
            Method::GET => get(handler),
            Method::POST => post(handler),
            _ => panic!("Unsupported method"),
        };

        Route {
            method,
            path,
            handler,
        }
    }
}

pub fn route<H, T, S>(
    _module: APIModule,
    path: &str,
    method: Method,
    handler: H,
) -> Router<SharedState>
where
    H: Handler<T, S> + Clone + Send + 'static,
    H::Future: Send,
    S: Send + Sync + Clone + 'static,
{
    let p: String = match path.trim_matches('/') {
        "" => "/".to_string(),
        p2 => format!("/{}/", p2),
    };
    let r = Route::new(path, method, handler);
    if p == "/" {
        Router::new().route(&p, r.handler.clone())
    } else {
        Router::new().route(&p, r.handler)
        // Router::new().route(&p, method_router.clone()).route(
        //     &p.clone().trim_end_matches("/"),
        //     get(move || async move { Redirect::permanent(&p) }),
        // )
    }
}
