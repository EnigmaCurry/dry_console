use crate::{api::route, app_state::SharedState, response::AppError};
use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
use hostname::get as host_name_get;
use semver::VersionReq;
use serde::Serialize;
use std::{ffi::OsStr, str::FromStr};
use strum::{AsRefStr, EnumIter, EnumProperty, EnumString, IntoEnumIterator};
use utoipa::ToSchema;
use which::which;

mod bash;
mod curl;
mod docker;
mod git;
mod htpasswd;
mod jq;
mod make;
mod openssl;
mod sed;
mod shred;
mod xargs;
mod xdg_open;

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
    #[strum(props(Version = "*"))]
    bash,
    #[strum(props(Version = "*"))]
    make,
    #[strum(props(Version = "*"))]
    sed,
    #[strum(props(Version = "*"))]
    xargs,
    #[strum(props(Version = "*"))]
    shred,
    #[strum(props(Version = "*"))]
    openssl,
    #[strum(props(Version = "*"))]
    htpasswd,
    #[strum(props(Version = "*"))]
    jq,
    #[strum(props(Version = "*", Name = "xdg-open"))]
    xdg_open,
    #[strum(props(Version = "*"))]
    curl,
}
impl WorkstationDependencies {
    fn get_version(&self) -> VersionReq {
        VersionReq::parse(self.get_str("Version").unwrap_or("*")).unwrap()
    }
    fn get_name(&self) -> &str {
        self.get_str("Name").unwrap_or_else(|| self.as_ref())
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
                name: dep.get_name().to_string(),
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
        match WorkstationDependencies::from_str(&name.clone().replace("-", "_")).ok() {
            Some(dependency) => {
                // Check if dependency is installed:
                let mut installed = false;
                let mut version = String::new();
                let path = match which(OsStr::new(dependency.get_name())) {
                    Ok(p) => {
                        installed = true;
                        p.to_string_lossy().to_string()
                    }
                    _ => String::new(),
                };
                if installed {
                    match dependency {
                        WorkstationDependencies::git => {
                            let v = git::get_version();
                            version = v;
                        }
                        WorkstationDependencies::docker => {
                            let v = docker::get_version();
                            version = v;
                        }
                        WorkstationDependencies::bash => {
                            let v = bash::get_version();
                            version = v;
                        }
                        WorkstationDependencies::make => {
                            let v = make::get_version();
                            version = v;
                        }
                        WorkstationDependencies::sed => {
                            let v = sed::get_version();
                            version = v;
                        }
                        WorkstationDependencies::xargs => {
                            let v = xargs::get_version();
                            version = v;
                        }
                        WorkstationDependencies::shred => {
                            let v = shred::get_version();
                            version = v;
                        }
                        WorkstationDependencies::openssl => {
                            let v = openssl::get_version();
                            version = v;
                        }
                        WorkstationDependencies::htpasswd => {
                            let v = htpasswd::get_version();
                            version = v;
                        }
                        WorkstationDependencies::jq => {
                            let v = jq::get_version();
                            version = v;
                        }
                        WorkstationDependencies::xdg_open => {
                            let v = xdg_open::get_version();
                            version = v;
                        }
                        WorkstationDependencies::curl => {
                            let v = curl::get_version();
                            version = v;
                        }
                    }
                };
                Json(WorkstationDependencyState {
                    name,
                    installed,
                    path,
                    version,
                })
            }
            .into_response(),
            None => AppError::Internal("Invalid dependency".to_string()).into_response(),
        }
    }
    route("/dependency/:name", get(handler))
}
