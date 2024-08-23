use crate::api::token::generate_deterministic_ulid_from_seed;
use dry_console_dto::script::ScriptEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::{Display, VariantNames};
use ulid::Ulid;

#[derive(VariantNames, Display)]
pub enum CommandLibrary {
    Hii,
}

pub fn new_script(variant: &str, script: &str) -> ScriptEntry {
    let id = generate_deterministic_ulid_from_seed(&variant);
    ScriptEntry {
        id,
        script: script.to_string(),
    }
}

impl CommandLibrary {
    pub fn get(&self) -> ScriptEntry {
        let variant = self.to_string();
        match self {
            CommandLibrary::Hii => new_script(
                &variant,
                r#"
                echo "Hii" >/dev/stderr
                for i in $(seq 100); do
                    echo $i
                    sleep 0.1
                done
                "#,
            ),
        }
    }
}
