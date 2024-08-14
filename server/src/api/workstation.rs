use crate::broadcast;
use crate::{api::route, app_state::SharedState, response::AppError};
use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
use dry_console_dto::workstation::{WorkstationDependencyInfo, WorkstationState, WorkstationUser};
use hostname::get as host_name_get;
use semver::VersionReq;
use serde::Serialize;
use std::{ffi::OsStr, str::FromStr};
use strum::{AsRefStr, EnumIter, EnumProperty, EnumString, IntoEnumIterator};
use utoipa::ToSchema;
use which::which;

pub mod command_execute;
mod dependencies;
pub mod platform;

pub fn router(shutdown: broadcast::Sender<()>) -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .merge(required_dependencies())
        .merge(dependencies())
        .merge(command_execute::main(shutdown))
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
    ssh,
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
        let platform = platform::detect_platform();
        Json(WorkstationState {
            hostname,
            user: WorkstationUser { uid, name },
            platform,
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
        match WorkstationDependencies::from_str(&name.clone().replace('-', "_")).ok() {
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
                            let v = dependencies::git::get_version();
                            version = v;
                        }
                        WorkstationDependencies::docker => {
                            let v = dependencies::docker::get_version();
                            version = v;
                        }
                        WorkstationDependencies::bash => {
                            let v = dependencies::bash::get_version();
                            version = v;
                        }
                        WorkstationDependencies::ssh => {
                            let v = dependencies::ssh::get_version();
                            version = v;
                        }
                        WorkstationDependencies::make => {
                            let v = dependencies::make::get_version();
                            version = v;
                        }
                        WorkstationDependencies::sed => {
                            let v = dependencies::sed::get_version();
                            version = v;
                        }
                        WorkstationDependencies::xargs => {
                            let v = dependencies::xargs::get_version();
                            version = v;
                        }
                        WorkstationDependencies::shred => {
                            let v = dependencies::shred::get_version();
                            version = v;
                        }
                        WorkstationDependencies::openssl => {
                            let v = dependencies::openssl::get_version();
                            version = v;
                        }
                        WorkstationDependencies::htpasswd => {
                            let v = dependencies::htpasswd::get_version();
                            version = v;
                        }
                        WorkstationDependencies::jq => {
                            let v = dependencies::jq::get_version();
                            version = v;
                        }
                        WorkstationDependencies::xdg_open => {
                            let v = dependencies::xdg_open::get_version();
                            version = v;
                        }
                        WorkstationDependencies::curl => {
                            let v = dependencies::curl::get_version();
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
