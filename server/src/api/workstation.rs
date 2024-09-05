use crate::broadcast;
use crate::{api::route, app_state::SharedState, response::AppError};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::{Request, State};
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
use utoipa::ToSchema;
use uzers::{get_current_uid, get_user_by_uid};
use which::which;

pub mod command;
pub mod command_execute;
pub mod d_rymcg_tech;
mod dependencies;
pub mod filesystem;
pub mod platform;

#[derive(Debug, Clone)]
pub enum WorkstationError {
    UnsupportedPlatform,
    UnsupportedDistribution,
}

pub fn router(shutdown: broadcast::Sender<()>, state: State<SharedState>) -> Router<SharedState> {
    Router::new()
        .merge(workstation())
        .merge(required_dependencies())
        .merge(dependencies())
        .merge(d_rymcg_tech::main())
        .merge(command::command())
        .merge(command_execute::main(shutdown, state))
        .merge(filesystem::validate_path())
}

#[allow(non_camel_case_types)]
#[derive(Serialize, EnumProperty, EnumString, EnumIter, AsRefStr)]
pub enum WorkstationDependency {
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

impl WorkstationDependency {
    fn get_version(&self) -> VersionReq {
        VersionReq::parse(self.get_str("Version").unwrap_or("*")).unwrap()
    }
    fn get_name(&self) -> &str {
        self.get_str("Name").unwrap_or_else(|| self.as_ref())
    }
}

#[derive(Clone, Debug, Default, Serialize, ToSchema)]
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
    async fn handler(State(state): State<SharedState>) -> impl IntoResponse {
        let hostname = host_name_get()
            .unwrap_or_else(|_| "Unknown".into())
            .to_string_lossy()
            .to_string();
        let uid = get_current_uid();
        let user = get_user_by_uid(get_current_uid()).unwrap();
        let name = user.name().to_string_lossy().to_string();
        let can_sudo;
        {
            can_sudo = state.read().await.sudo_enabled;
        }
        let the_platform = platform::detect_platform();
        Json(WorkstationState {
            hostname,
            user: WorkstationUser {
                uid,
                name,
                can_sudo,
            },
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
    async fn handler(State(state): State<SharedState>) -> impl IntoResponse {
        let deps: Vec<WorkstationDependencyInfo> = WorkstationDependency::iter()
            .map(|dep| WorkstationDependencyInfo {
                name: dep.get_name().to_string(),
                version: dep.get_version().to_string(),
                packages: dependencies::bash::get_packages(platform::detect_platform())
                    .expect("failed to get package definitions"),
            })
            .collect();
        let mut state = state.write().await;
        state.missing_dependencies.clear();
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
    async fn handler(
        Path(name): Path<String>,
        State(state): State<SharedState>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        match WorkstationDependency::from_str(&name.clone().replace('-', "_")).ok() {
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
                        WorkstationDependency::git => {
                            version = dependencies::git::get_version();
                        }
                        WorkstationDependency::docker => {
                            version = dependencies::docker::get_version();
                        }
                        WorkstationDependency::bash => {
                            version = dependencies::bash::get_version();
                        }
                        WorkstationDependency::ssh => {
                            version = dependencies::ssh::get_version();
                        }
                        WorkstationDependency::make => {
                            version = dependencies::make::get_version();
                        }
                        WorkstationDependency::sed => {
                            version = dependencies::sed::get_version();
                        }
                        WorkstationDependency::xargs => {
                            version = dependencies::xargs::get_version();
                        }
                        WorkstationDependency::shred => {
                            version = dependencies::shred::get_version();
                        }
                        WorkstationDependency::openssl => {
                            version = dependencies::openssl::get_version();
                        }
                        WorkstationDependency::htpasswd => {
                            version = dependencies::htpasswd::get_version();
                        }
                        WorkstationDependency::jq => {
                            version = dependencies::jq::get_version();
                        }
                        WorkstationDependency::xdg_open => {
                            version = dependencies::xdg_open::get_version();
                        }
                        WorkstationDependency::curl => {
                            version = dependencies::curl::get_version();
                        }
                    }
                }
                match dependency {
                    WorkstationDependency::git => {
                        packages.as_mut().unwrap().extend(
                            dependencies::git::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::docker => {
                        packages.as_mut().unwrap().extend(
                            dependencies::docker::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::bash => {
                        packages.as_mut().unwrap().extend(
                            dependencies::bash::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::ssh => {
                        packages.as_mut().unwrap().extend(
                            dependencies::ssh::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::make => {
                        packages.as_mut().unwrap().extend(
                            dependencies::make::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::sed => {
                        packages.as_mut().unwrap().extend(
                            dependencies::sed::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::xargs => {
                        packages.as_mut().unwrap().extend(
                            dependencies::xargs::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::shred => {
                        packages.as_mut().unwrap().extend(
                            dependencies::shred::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::openssl => {
                        packages.as_mut().unwrap().extend(
                            dependencies::openssl::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::htpasswd => {
                        packages.as_mut().unwrap().extend(
                            dependencies::htpasswd::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::jq => {
                        packages.as_mut().unwrap().extend(
                            dependencies::jq::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::xdg_open => {
                        packages.as_mut().unwrap().extend(
                            dependencies::xdg_open::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                    WorkstationDependency::curl => {
                        packages.as_mut().unwrap().extend(
                            dependencies::curl::get_packages(platform)
                                .expect("did not get package definitions"),
                        );
                    }
                }
                let dep_state = WorkstationDependencyState {
                    name,
                    installed,
                    path,
                    version,
                    packages: packages.expect("failed to find packages definition"),
                };
                if !installed {
                    let mut state = state.write().await;
                    state.missing_dependencies.push(dep_state.clone());
                    //debug!("missing_dependencies: {:?}", state.missing_dependencies);
                }
                Json(dep_state)
            }
            .into_response(),
            None => AppError::Internal(anyhow!("Invalid dependency"), Some(req.uri().to_string()))
                .into_response(),
        }
    }
    route("/dependency/:name/", get(handler))
}
