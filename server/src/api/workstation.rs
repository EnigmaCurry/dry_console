use crate::broadcast;
use crate::{api::route, app_state::SharedState, response::AppError};
use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};
pub use dry_console_dto::workstation::{
    WorkstationDependencyInfo, WorkstationPackage, WorkstationPackageManager, WorkstationState,
    WorkstationUser,
};
use hostname::get as host_name_get;
use semver::VersionReq;
use serde::Serialize;
use std::{ffi::OsStr, str::FromStr};
use strum::{AsRefStr, EnumIter, EnumProperty, EnumString, IntoEnumIterator};
use tracing::debug;
use utoipa::ToSchema;
use uzers::{get_current_uid, get_user_by_uid};
use which::which;

pub mod command;
pub mod command_execute;
mod dependencies;
pub mod platform;

#[derive(Debug, Clone)]
pub enum WorkstationError {
    UnknownDependency,
    UnsupportedPlatform,
    UnsupportedDistribution,
}

pub fn router(shutdown: broadcast::Sender<()>) -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .merge(required_dependencies())
        .merge(dependencies())
        .merge(command::command())
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
    /// List of required packages to install for this dependency.
    packages: Vec<WorkstationPackage>,
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
        let uid = get_current_uid();
        let user = get_user_by_uid(get_current_uid()).unwrap();
        let name = user.name().to_string_lossy().to_string();
        let the_platform = platform::detect_platform();
        Json(WorkstationState {
            hostname,
            user: WorkstationUser { uid, name },
            platform: the_platform,
        })
        .into_response()
    }
    route("/", get(handler))
}

#[utoipa::path(
    get,
    path = "/api/workstation/dependencies/",
    responses(
        (status = OK, description = "Required dependencies")
    ),
)]
fn required_dependencies() -> Router<SharedState> {
    async fn handler() -> impl IntoResponse {
        let deps: Vec<WorkstationDependencyInfo> = WorkstationDependencies::iter()
            .map(|dep| WorkstationDependencyInfo {
                name: dep.get_name().to_string(),
                version: dep.get_version().to_string(),
                packages: dependencies::bash::get_packages(platform::detect_platform())
                    .expect("failed to get package definitions"),
            })
            .collect();
        Json(&deps).into_response()
    }
    route("/dependencies/", get(handler))
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
                let mut packages: Result<Vec<WorkstationPackage>, WorkstationError> =
                    Ok(Vec::<WorkstationPackage>::new());
                let path = match which(OsStr::new(dependency.get_name())) {
                    Ok(p) => {
                        installed = true;
                        p.to_string_lossy().to_string()
                    }
                    _ => String::new(),
                };
                let platform = platform::detect_platform();
                if installed {
                    match dependency {
                        WorkstationDependencies::git => {
                            version = dependencies::git::get_version();
                        }
                        WorkstationDependencies::docker => {
                            version = dependencies::docker::get_version();
                        }
                        WorkstationDependencies::bash => {
                            version = dependencies::bash::get_version();
                        }
                        WorkstationDependencies::ssh => {
                            version = dependencies::ssh::get_version();
                        }
                        WorkstationDependencies::make => {
                            version = dependencies::make::get_version();
                        }
                        WorkstationDependencies::sed => {
                            version = dependencies::sed::get_version();
                        }
                        WorkstationDependencies::xargs => {
                            version = dependencies::xargs::get_version();
                        }
                        WorkstationDependencies::shred => {
                            version = dependencies::shred::get_version();
                        }
                        WorkstationDependencies::openssl => {
                            version = dependencies::openssl::get_version();
                        }
                        WorkstationDependencies::htpasswd => {
                            version = dependencies::htpasswd::get_version();
                        }
                        WorkstationDependencies::jq => {
                            version = dependencies::jq::get_version();
                        }
                        WorkstationDependencies::xdg_open => {
                            version = dependencies::xdg_open::get_version();
                        }
                        WorkstationDependencies::curl => {
                            version = dependencies::curl::get_version();
                        }
                    }
                };
                match dependency {
                    WorkstationDependencies::git => {
                        packages.as_mut().unwrap().extend(
                            dependencies::git::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::docker => {
                        packages.as_mut().unwrap().extend(
                            dependencies::docker::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::bash => {
                        packages.as_mut().unwrap().extend(
                            dependencies::bash::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::ssh => {
                        packages.as_mut().unwrap().extend(
                            dependencies::ssh::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::make => {
                        packages.as_mut().unwrap().extend(
                            dependencies::make::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::sed => {
                        packages.as_mut().unwrap().extend(
                            dependencies::sed::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::xargs => {
                        packages.as_mut().unwrap().extend(
                            dependencies::xargs::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::shred => {
                        packages.as_mut().unwrap().extend(
                            dependencies::shred::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::openssl => {
                        packages.as_mut().unwrap().extend(
                            dependencies::openssl::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::htpasswd => {
                        packages.as_mut().unwrap().extend(
                            dependencies::htpasswd::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::jq => {
                        packages.as_mut().unwrap().extend(
                            dependencies::jq::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::xdg_open => {
                        packages.as_mut().unwrap().extend(
                            dependencies::xdg_open::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependencies::curl => {
                        packages.as_mut().unwrap().extend(
                            dependencies::curl::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                }
                Json(WorkstationDependencyState {
                    name,
                    installed,
                    path,
                    version,
                    packages: packages.expect("failed to find packages definition"),
                })
            }
            .into_response(),
            None => AppError::Internal("Invalid dependency".to_string()).into_response(),
        }
    }
    route("/dependency/:name/", get(handler))
}
