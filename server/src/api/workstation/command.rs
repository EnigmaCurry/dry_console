use crate::response::{AppError, AppJson, JsonResult};
use crate::{api::token::generate_deterministic_ulid_from_seed, routing::route, AppRouter};
use axum::{extract::Path, routing::get};
use dry_console_dto::script::ScriptEntry;
use indoc::indoc;
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumString, VariantNames};

#[derive(EnumString, VariantNames, Display, AsRefStr)]
pub enum CommandLibrary {
    TestExampleOne,
}

pub fn new_script(variant: &str, description: &str, script: &str) -> ScriptEntry {
    let id = generate_deterministic_ulid_from_seed(variant);
    ScriptEntry {
        id,
        description: description.to_string(),
        script: script.to_string(),
    }
}

impl CommandLibrary {
    pub fn get(&self) -> ScriptEntry {
        let variant = self.to_string();
        match self {
            CommandLibrary::TestExampleOne => {
                let filename = format!("./scripts/{}.sh", self.as_ref());
                let script = include_str!(filename);
                new_script(&variant, "Count to 100", &script)
            }
        }
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
                let script_entry = command.get();
                Ok(AppJson(script_entry))
            }
            Err(_) => Err(AppError::NotFound),
        }
    }
    route("/command/:command", get(handler))
}
