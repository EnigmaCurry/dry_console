use regex::Regex;
use std::process::Command;

pub fn get_version() -> String {
    // AFAIK there is no direct way to check the version of htpasswd, so just hard code it:
    "unknown".to_string()
}
