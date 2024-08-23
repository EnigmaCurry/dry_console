use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptEntry {
    pub id: Ulid,
    pub script: String,
}
