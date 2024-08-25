use crate::api::token::generate_deterministic_ulid_from_seed;
use crate::response::{AppError, AppJson, JsonResult};
use crate::{routing::route, AppRouter};
use axum::{extract::Path, routing::get};
pub use dry_console_dto::script::ScriptEntry;
pub use dry_console_script::CommandLibrary;
use std::str::FromStr;

pub trait CommandLibraryExt {
    fn get_script(&self) -> &'static str;
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
                let script = command.get_script();
                let script_entry = ScriptEntry {
                    id: generate_deterministic_ulid_from_seed(script),
                    description: "missing description".to_string(),
                    script: script.to_string(),
                };
                Ok(AppJson(script_entry))
            }
            Err(_) => Err(AppError::NotFound),
        }
    }
    route("/command/:command", get(handler))
}
