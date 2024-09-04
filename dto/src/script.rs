use dry_console_common::token::generate_deterministic_ulid_from_seed;
use indoc::indoc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct ScriptEntry {
    pub id: Ulid,
    pub description: String,
    pub script: String,
    pub env: Vec<EnvVarDescription>,
}

impl Default for ScriptEntry {
    fn default() -> Self {
        let script = indoc! {"
            ## Failed to find command in Command Library

            echo \"Failed to find command in Command Library && exit 1\"
        "};
        let id = Ulid::default();
        let (description, script, env) = extract_source_and_description(script)
            .expect("error parsing shell script source and/or description");
        Self {
            id,
            description,
            script,
            env,
        }
    }
}

impl ScriptEntry {
    pub fn from_source(source: String) -> Self {
        let id = generate_deterministic_ulid_from_seed(&source);
        let (description, script, env) = extract_source_and_description(&source)
            .expect("error parsing shell script source / description");
        Self {
            id,
            description,
            script,
            env,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EnvVarType {
    String,
    List,
    Bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EnvVarDescription {
    pub name: String,
    pub description: String,
    pub var_type: EnvVarType,
    pub default_value: String,
}

#[allow(clippy::manual_strip)]
fn trim_single_starting_space(line: &str) -> &str {
    if line.starts_with(' ') {
        &line[1..]
    } else {
        line
    }
}

fn parse_env_var_directive(line: &str) -> Option<EnvVarDescription> {
    // Define the regex pattern
    let pattern = r#"# var:\s*(\w+)\s*(?:=\s*("(?:[^"]*)"|[^\s]+))?\s*(\([^\)]*\))?\s*(.*)"#;
    let re = Regex::new(pattern).unwrap();

    // Check if the line matches the regex
    if let Some(captures) = re.captures(line) {
        // Extract the variable name
        let name = captures.get(1)?.as_str().to_string();

        // Extract the default value (if any), and remove surrounding quotes for quoted values
        let default_value = captures
            .get(2)
            .map_or(String::new(), |m| m.as_str().trim_matches('"').to_string());

        // Extract the type (if any), and remove parentheses
        let var_type_str = captures
            .get(3)
            .map_or("", |m| m.as_str().trim_matches(|p| p == '(' || p == ')'));
        let var_type = match var_type_str {
            "bool" | "Bool" => EnvVarType::Bool,
            "list" | "List" => EnvVarType::List,
            "string" | "String" => EnvVarType::String,
            _ => EnvVarType::String, // Default to String if no valid type is found
        };

        // Extract the description (if any)
        let description = captures
            .get(4)
            .map_or(String::new(), |m| m.as_str().to_string());

        // Return the parsed values as an EnvVarDescription struct
        Some(EnvVarDescription {
            name,
            description,
            var_type,
            default_value,
        })
    } else {
        None
    }
}

fn extract_source_and_description(
    script: &str,
) -> Option<(String, String, Vec<EnvVarDescription>)> {
    let mut description = Vec::new();
    let mut stripped_script = String::new();
    let mut env_vars = Vec::new();
    let mut in_description = true;

    for line in script.lines() {
        if let Some(env_var) = parse_env_var_directive(line) {
            env_vars.push(env_var);
            continue;
        }

        if in_description {
            if line.starts_with('#') {
                let comment_content = trim_single_starting_space(line.trim_start_matches('#'));
                description.push(comment_content.to_string());
            } else {
                in_description = false;
                stripped_script.push_str(line);
                stripped_script.push('\n');
            }
        } else {
            stripped_script.push_str(line);
            stripped_script.push('\n');
        }
    }

    if !description.is_empty() {
        Some((
            description.join("\n"),
            stripped_script.trim_start().to_string(),
            env_vars,
        ))
    } else {
        None
    }
}
