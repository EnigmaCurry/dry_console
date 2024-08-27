use crate::api::token::generate_deterministic_ulid_from_seed;
use crate::response::{AppError, AppJson, JsonResult};
use crate::COMMAND_LIBRARY_MAP;
use crate::{routing::route, AppRouter};
use axum::{extract::Path, routing::get};
pub use dry_console_dto::script::ScriptEntry;
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter, EnumString, VariantNames};
use ulid::Ulid;

#[derive(EnumString, VariantNames, Display, AsRefStr, EnumIter, PartialEq, Debug, Clone)]
pub enum CommandLibrary {
    TestExampleOne,
    InstallDRymcgTech,
}
impl CommandLibrary {
    pub fn from_id(id: Ulid) -> Option<Self> {
        COMMAND_LIBRARY_MAP.get(&id.to_string()).cloned()
    }
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
    async fn handler(Path(command): Path<String>) -> JsonResult<ScriptEntry> {
        match CommandLibrary::from_str(&command) {
            Ok(command) => {
                let script_entry = ScriptEntry::from_source(command.get_script());
                Ok(AppJson(script_entry))
            }
            Err(_) => Err(AppError::NotFound),
        }
    }
    route("/command/:command", get(handler))
}
