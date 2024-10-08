use crate::app_state::SharedState;
use crate::response::{AppError, AppJson, JsonResult};
use crate::{routing::route, AppRouter};
use axum::extract::State;
use axum::{extract::Path, routing::get};
pub use dry_console_dto::script::ScriptEntry;
use dry_console_dto::workstation::{Distribution, WorkstationPackageManager};
use indoc::formatdoc;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter, EnumString, VariantNames};
use ulid::Ulid;

use super::WorkstationDependencyState;

#[derive(
    EnumString, VariantNames, Display, AsRefStr, EnumIter, PartialEq, Debug, Clone, Hash, Eq,
)]
pub enum CommandLibrary {
    TestExampleOne,
    InstallDependencies,
    InstallDRymcgTech,
}
impl CommandLibrary {
    pub async fn from_id(
        id: Ulid,
        command_library: HashMap<String, CommandLibrary>,
    ) -> Option<Self> {
        command_library.get(&id.to_string()).cloned()
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

    let mut commands = Vec::<String>::new();
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
                let distribution;
                {
                    let state = state.write().await;
                    distribution = state.platform.distribution.clone();
                }
                let package_manager = match distribution {
                    Distribution::Fedora => WorkstationPackageManager::Dnf,
                    _ => {
                        return Err(AppError::Internal(
                            "Unimplemented package manager for InstallDependencies script:"
                                .to_string(),
                        ))
                    }
                };
                let script;
                {
                    let state = state.read().await;
                    script = generate_install_commands(&state.missing_dependencies);
                }
                let script_entry = ScriptEntry::from_source(formatdoc! {"
                    # # Install missing dependencies                    
                    # This script is customized for {distribution} ({package_manager} package manager).
                    {script}
                "});
                {
                    let mut state = state.write().await;
                    // debug!(
                    //     "Inserting new command overlay: {}",
                    //     script_entry.id.to_string()
                    // );
                    state.command_id.insert(
                        CommandLibrary::InstallDependencies,
                        script_entry.id.to_string(),
                    );
                    state
                        .command_script
                        .insert(script_entry.id.to_string(), script_entry.clone().script);
                    state.command_library.insert(
                        script_entry.id.to_string(),
                        CommandLibrary::InstallDependencies,
                    );
                }
                Ok(AppJson(script_entry))
            }
            _ => match CommandLibrary::from_str(&command) {
                // No special handling, return the static script:
                Ok(command) => {
                    let state = state.read().await;
                    let script_entry = ScriptEntry::from_source(
                        command.get_script(&state.command_id, &state.command_script),
                    );
                    Ok(AppJson(script_entry))
                }
                Err(_) => Err(AppError::NotFound),
            },
        }
    }
    route("/command/:command", get(handler))
}
