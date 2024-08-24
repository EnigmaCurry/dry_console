use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ScriptEntry {
    pub id: Ulid,
    pub description: String,
    pub script: String,
}
