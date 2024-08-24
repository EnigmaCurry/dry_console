use crate::api::token::generate_deterministic_ulid_from_seed;
use dry_console_dto::script::ScriptEntry;
use strum::{Display, VariantNames};

#[derive(VariantNames, Display)]
pub enum CommandLibrary {
    TestExampleOne,
}

pub fn new_script(variant: &str, script: &str) -> ScriptEntry {
    let id = generate_deterministic_ulid_from_seed(variant);
    ScriptEntry {
        id,
        script: script.to_string(),
    }
}

impl CommandLibrary {
    pub fn get(&self) -> ScriptEntry {
        let variant = self.to_string();
        match self {
            CommandLibrary::TestExampleOne => new_script(
                &variant,
                r#"
                echo "Hii" >/dev/stderr
                for i in $(seq 100); do
                    echo $i
                    #sleep 0.1
                done
                "#,
            ),
        }
    }
}
