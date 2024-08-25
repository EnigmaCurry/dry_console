use dry_console_dto::script::ScriptEntry;
use sha2::{Digest, Sha256};
use strum::{AsRefStr, Display, EnumIter, EnumString, VariantNames};
use ulid::Ulid;

#[derive(EnumString, VariantNames, Display, AsRefStr, EnumIter)]
pub enum CommandLibrary {
    TestExampleOne,
}

fn generate_deterministic_ulid_from_seed(seed: &str) -> Ulid {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    let result = hasher.finalize();
    // Use the first 16 bytes of the hash to create a ULID
    let bytes = &result[..16];
    Ulid::from_bytes(bytes.try_into().expect("slice with incorrect length"))
}

pub fn new_script(variant: &str, description: &str, script: &str) -> ScriptEntry {
    let id = generate_deterministic_ulid_from_seed(variant);
    ScriptEntry {
        id,
        description: description.to_string(),
        script: script.to_string(),
    }
}
