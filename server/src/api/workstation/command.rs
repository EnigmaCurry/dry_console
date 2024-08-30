use crate::app_state::{AppState, SharedState};
use crate::response::{AppError, AppJson, JsonResult};
use crate::COMMAND_LIBRARY_MAP;
use crate::{routing::route, AppRouter};
use axum::extract::State;
use axum::{extract::Path, routing::get};
use dry_console_common::token::generate_deterministic_ulid_from_seed;
pub use dry_console_dto::script::ScriptEntry;
use dry_console_dto::workstation::{Distribution, WorkstationPackageManager};
use indoc::formatdoc;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use strum::{AsRefStr, Display, EnumIter, EnumString, VariantNames};
use tracing::debug;
use ulid::Ulid;

use super::{WorkstationDependency, WorkstationDependencyState};

#[derive(EnumString, VariantNames, Display, AsRefStr, EnumIter, PartialEq, Debug, Clone)]
pub enum CommandLibrary {
    TestExampleOne,
    InstallDependencies,
    InstallDRymcgTech,
}
impl CommandLibrary {
    pub fn from_id(id: Ulid) -> Option<Self> {
        COMMAND_LIBRARY_MAP.get(&id.to_string()).cloned()
    }
}
fn generate_install_commands(uninstalled_dependencies: &[WorkstationDependencyState]) -> String {
    let mut package_map: HashMap<&str, HashSet<String>> = HashMap::new();

    for dep in uninstalled_dependencies {
        for pkg in &dep.packages {
            let manager = match pkg.package_manager.clone() {
                WorkstationPackageManager::Dnf => "dnf",
                WorkstationPackageManager::Pacman => "pacman",
                WorkstationPackageManager::Apt => "apt",
                WorkstationPackageManager::Apk => "apk",
            };
            package_map
                .entry(manager)
                .or_default()
                .insert(pkg.package_name.clone());
        }
    }

    let mut commands = Vec::new();
    if let Some(packages) = package_map.get("dnf") {
        commands.push(format!(
            "sudo dnf install -y {}",
            packages
                .iter()
                .sorted()
                .cloned()
                .collect::<Vec<String>>()
                .join(" ")
        ));
    }
    if let Some(packages) = package_map.get("pacman") {
        commands.push(format!(
            "sudo pacman -S --noconfirm {}",
            packages
                .iter()
                .sorted()
                .cloned()
                .collect::<Vec<String>>()
                .join(" ")
        ));
    }
    if let Some(packages) = package_map.get("apt") {
        commands.push(format!(
            "DEBIAN_FRONTEND=noninteractive apt-get install -y {}",
            packages
                .iter()
                .sorted()
                .cloned()
                .collect::<Vec<String>>()
                .join(" ")
        ));
    }
    if let Some(packages) = package_map.get("apk") {
        commands.push(format!(
            "apk add --no-confirm {}",
            packages
                .iter()
                .sorted()
                .cloned()
                .collect::<Vec<String>>()
                .join(" ")
        ));
    }

    commands.join("\n")
}

#[utoipa::path(
    get,
    path = "/api/workstation/command/{command}/",
    responses(
        (status = OK, body = ScriptEntry, description = "Get details about a command from the library"),
        (status = NOT_FOUND, description = "Command not found in the library")
    ),
    params(
        ("command" = String, Path, description = "The name (id) of the command to retrieve")
    )
)]
pub fn command() -> AppRouter {
    async fn handler(
        Path(command): Path<String>,
        State(state): State<SharedState>,
    ) -> JsonResult<ScriptEntry> {
        // Special handling for scripts by name:
        match command.as_str() {
            "InstallDependencies" => {
                let mut state = state.write().await;
                let distribution = state.platform.distribution.clone();
                let package_manager = match distribution {
                    Distribution::Fedora => WorkstationPackageManager::Dnf,
                    _ => {
                        return Err(AppError::Internal(
                            "Unimplemented package manager for InstallDependencies script:"
                                .to_string(),
                        ))
                    }
                };
                let script = generate_install_commands(&state.missing_dependencies);
                let script_entry = ScriptEntry::from_source(formatdoc! {"
                    # # Install missing dependencies
                    
                    # This script is customized for {distribution} ({package_manager} package manager).
                    {script}
                "});
                state
                    .command_library_overlay
                    .insert(script_entry.id.to_string(), script_entry.clone().script);

                Ok(AppJson(script_entry))
            }
            _ => match CommandLibrary::from_str(&command) {
                // No special handling, return the static script:
                Ok(command) => {
                    let state = state.read().await;
                    let script_entry = ScriptEntry::from_source(
                        command.get_script(&state.command_library_overlay),
                    );
                    Ok(AppJson(script_entry))
                }
                Err(_) => Err(AppError::NotFound),
            },
        }
    }
    route("/command/:command", get(handler))
}
