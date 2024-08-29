use dry_console_common::token::generate_deterministic_ulid_from_seed;
use indoc::indoc;
use serde::{Deserialize, Serialize};
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
        let script = indoc! {"
            ## Failed to find command in Command Library

            echo \"Failed to find command in Command Library && exit 1\"
        "};
        let id = Ulid::default();
        let (description, script) = extract_source_and_description(script)
            .expect("error parsing shell script source and/or description");
        Self {
            id,
            description,
            script,
        }
    }
}

impl ScriptEntry {
    pub fn from_source(source: String) -> Self {
        let id = generate_deterministic_ulid_from_seed(&source);
        let (description, script) = extract_source_and_description(&source)
            .expect("error parsing shell script source / description");
        Self {
            id,
            description,
            script,
        }
    }
}

#[allow(clippy::manual_strip)]
fn trim_single_starting_space(line: &str) -> &str {
    if line.starts_with(' ') {
        &line[1..]
    } else {
        line
    }
}

fn extract_source_and_description(script: &str) -> Option<(String, String)> {
    let mut description = Vec::new();
    let mut stripped_script = String::new();
    let mut in_description = true;

    for line in script.lines() {
        if in_description {
            if line.starts_with('#') {
                let comment_content = trim_single_starting_space(line.trim_start_matches('#'));
                description.push(comment_content.to_string());
            } else {
                // End description when the first non-comment, non-empty line is encountered
                in_description = false;
                stripped_script.push_str(line);
                stripped_script.push('\n');
            }
        } else {
            // After the description ends, include all lines in the script
            stripped_script.push_str(line);
            stripped_script.push('\n');
        }
    }

    if !description.is_empty() {
        Some((
            description.join("\n"),
            stripped_script.trim_start().to_string(),
        ))
    } else {
        None
    }
}
