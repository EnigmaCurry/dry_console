use crate::{api::route, app_state::SharedState, response::AppError};
use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
use hostname::get as host_name_get;
use hyper::StatusCode;
use semver::VersionReq;
use serde::Serialize;
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumProperty, EnumString, IntoEnumIterator};
use utoipa::ToSchema;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .merge(required_dependencies())
        .merge(dependencies())
}

#[derive(Default, Serialize, ToSchema)]
pub struct WorkstationUser {
    uid: u32,
    name: String,
}

#[derive(Default, Serialize, ToSchema)]
pub struct WorkstationState {
    /// Hostname of the workstation.
    hostname: String,
    user: WorkstationUser,
}

#[derive(Serialize)]
struct WorkstationDependencyInfo {
    name: String,
    version: String,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, EnumProperty, EnumString, EnumIter, AsRefStr)]
pub enum WorkstationDependencies {
    #[strum(props(Version = "*"))]
    git,
    #[strum(props(Version = "*"))]
    docker,
}
impl WorkstationDependencies {
    fn get_version(&self) -> VersionReq {
        // Retrieve the version property using strum's get_str method
        VersionReq::parse(self.get_str("Version").unwrap()).unwrap()
    }
}

#[derive(Default, Serialize, ToSchema)]
pub struct WorkstationDependencyState {
    /// Name of the dependency.
    name: String,
    /// Whether or not the dependency is installed.
    installed: bool,
    /// Path of the installed dependency.
    path: String,
    /// Version of installed dependency.
    version: String,
}

#[utoipa::path(
    get,
    path = "/api/workstation/",
    responses(
        (status = OK, description = "Workstation info", body = WorkstationState)
    ),
)]
fn workstation() -> Router<SharedState> {
    async fn handler() -> impl IntoResponse {
        let hostname = host_name_get()
            .unwrap_or_else(|_| "Unknown".into())
            .to_string_lossy()
            .to_string();
        let uid = users::get_current_uid();
        let user = users::get_user_by_uid(users::get_current_uid()).unwrap();
        let name = user.name().to_string_lossy().to_string();
        Json(WorkstationState {
            hostname,
            user: WorkstationUser { uid, name },
        })
        .into_response()
    }
    route("/", get(handler))
}

#[utoipa::path(
    get,
    path = "/api/workstation/dependencies",
    responses(
        (status = OK, description = "Required dependencies")
    ),
)]
fn required_dependencies() -> Router<SharedState> {
    async fn handler() -> impl IntoResponse {
        let dependencies: Vec<WorkstationDependencyInfo> = WorkstationDependencies::iter()
            .map(|dep| WorkstationDependencyInfo {
                name: dep.as_ref().to_string(),
                version: dep.get_version().to_string(),
            })
            .collect();
        Json(&dependencies).into_response()
    }
    route("/dependencies", get(handler))
}

#[utoipa::path(
    get,
    path = "/api/workstation/dependency/{name}",
    responses(
        (status = OK, description = "Workstation info", body = WorkstationDependencyState)
    ),
    params(
        ("name" = str, Path, description="Dependency name"),
    )
)]
fn dependencies() -> Router<SharedState> {
    async fn handler(Path(name): Path<String>) -> impl IntoResponse {
        fn match_name_to_dependency(name: String) -> Option<WorkstationDependencies> {
            WorkstationDependencies::from_str(&name).ok()
        }
        match match_name_to_dependency(name.clone()) {
            Some(_dependency) => Json(WorkstationDependencyState {
                name,
                installed: false,
                path: "".to_string(),
                version: "x.x.x".to_string(),
            })
            .into_response(),
            None => AppError::Internal("Invalid dependency".to_string()).into_response(),
        }
    }
    route("/dependency/:name", get(handler))
}
