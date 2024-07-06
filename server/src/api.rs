use axum::routing::get;
use axum::Router;
use axum::{response::Redirect, routing::MethodRouter};
use enum_iterator::{all, Sequence};

mod test;
mod workstation;

const API_PREFIX: &str = "api";

/// All API modules (and sub-modules) must implement ApiModule trait:
pub trait ApiModule {
    fn main() -> Router;
    fn to_string(&self) -> String;
    fn router(&self) -> Router;
    #[allow(dead_code)]
    fn redirect(&self) -> MethodRouter;
}

/// Enumeration of all top-level modules:
#[derive(Debug, PartialEq, Sequence, Clone)]
pub enum APIModule {
    Test,
    Workstation,
}
impl ApiModule for APIModule {
    fn main() -> Router {
        // Adds all routes for all modules in APIModule:
        let mut app = Router::new();
        for m in all::<APIModule>() {
            app = app.merge(m.router());
        }
        app
    }
    fn router(&self) -> Router {
        match self {
            APIModule::Test => test::router(),
            APIModule::Workstation => workstation::router(),
        }
    }
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
    fn redirect(&self) -> MethodRouter {
        let r = format!("/{}/{}/", API_PREFIX, self.to_string());
        get(move || async move { Redirect::permanent(&r) })
    }
}

pub fn router() -> Router {
    // Adds all routes for all modules in APIModule:
    let r = APIModule::main();
    //tracing::debug!("{r:#?}");
    r
}

fn mod_route(module: APIModule, path: &str, method_router: MethodRouter<()>) -> Router {
    let path_stripped = path.trim_matches('/');
    let path = format!("{path_stripped}/");
    Router::new().route(
        format!("/{}/{}/{}", API_PREFIX, module.to_string(), path).as_str(),
        method_router,
    )
}
