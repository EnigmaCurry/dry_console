use crate::{
    api::{route, APIModule},
    app_state::SharedState,
};
use axum::{response::Redirect, routing::get, Json, Router};
//use serde_json::json;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

#[utoipauto(paths = "./server/src")]
#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(SwaggerUi::new("/ui").url("/api/docs/openapi.json/", ApiDoc::openapi()))
        .merge(docs())
        .merge(ui())
        .with_state(SharedState::default())
}

#[utoipa::path(
    get,
    path = "/api/docs/openapi.json/",
    responses(
        (status = 200, description = "OpenAPI spec JSON file", body = ())
    )
)]
async fn handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

fn docs() -> Router<SharedState> {
    route("/openapi.json", get(handler))
}

fn ui() -> Router<SharedState> {
    route("/", get(|| async { Redirect::permanent("/api/docs/ui/") }))
}
