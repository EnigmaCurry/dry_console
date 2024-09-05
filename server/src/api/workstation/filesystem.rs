use crate::app_state::SharedState;
use crate::path::{
    could_create_path, directory_is_writable_by_user, expand_tilde, find_nearest_existing_parent,
};
use crate::response::{AppError, AppJson, JsonResult};
use crate::{routing::route, AppRouter};
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::{Query, RawPathParams, Request, State};
use axum::{extract::Path, routing::get};
use axum::{Json, Router};
pub use dry_console_dto::script::ScriptEntry;
use dry_console_dto::workstation::{Distribution, PathValidationResult, WorkstationPackageManager};
use indoc::formatdoc;
use itertools::Itertools;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;
use strum::{AsRefStr, Display, EnumIter, EnumString, VariantNames};
use ulid::Ulid;

use super::WorkstationDependencyState;

#[derive(Deserialize)]
struct PathParams {
    path: String,
}
#[utoipa::path(
    get,
    path = "/api/workstation/filesystem/validate_path/",
    responses(
        (status = OK, body = PathValidationResult, description = "Get details about the filesystem path"),
        (status = NOT_FOUND, description = "Command not found in the library")
    ),
    params(
        ("path" = String, Query, description = "The workstation path to validate")
    )
)]
pub fn validate_path() -> Router<SharedState> {
    pub async fn handler(Query(params): Query<PathParams>) -> Json<PathValidationResult> {
        let mut path = expand_tilde(&params.path.strip_suffix("/").unwrap_or(&params.path));

        let mut result = PathValidationResult {
            path: path.clone(),
            exists: path.exists(),
            writable: false,
            is_directory: path.is_dir(),
            can_be_created: could_create_path(path.as_path()).is_ok(),
        };

        // Check if the path exists and is writable
        if result.exists {
            result.writable = directory_is_writable_by_user(&path);
        } else {
            result.can_be_created = could_create_path(&path).is_ok();
        }

        // Ensure path has a trailing slash if it's a directory
        if result.is_directory {
            let mut path_str = path.to_string_lossy().to_string();
            if !path_str.ends_with('/') {
                path_str.push('/');
            }
            result.path = PathBuf::from(path_str);
        }

        Json(result)
    }

    route("/filesystem/validate_path", get(handler))
}
