use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct ScriptEntry {
    pub id: Ulid,
    pub description: String,
    pub script: String,
}

impl Default for ScriptEntry {
    fn default() -> Self {
        Self {
            id: Ulid::default(),
            description: "Failed to find command in Command Library".to_string(),
            script: "echo 'Failed to find command in Command Library' && exit 1".to_string(),
        }
    }
}
