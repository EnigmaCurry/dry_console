use crate::{
    api::{route, APIModule},
    app_state::SharedState,
};
use axum::{http::Method, routing::get, Json, Router};
//use serde_json::json;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(handler))]
struct ApiDoc;

pub fn router() -> Router<SharedState> {
    let mut doc = ApiDoc::openapi();
    Router::new()
        .merge(SwaggerUi::new("/ui").url("/api/docs/openapi.json", doc))
        .merge(docs())
        .with_state(SharedState::default())
}

#[utoipa::path(
    get,
    path = "/api/docs/openapi.json/",
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
async fn handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

fn docs() -> Router<SharedState> {
    route(APIModule::Docs, "/openapi.json", Method::GET, get(handler))
}

// fn workstation_dependencies() -> Router<SharedState> {
//     async fn handler() -> &'static str {
//         "Workstation foo"
//     }
//     route(APIModule::Workstation, "/dependencies", get(handler))
// }
