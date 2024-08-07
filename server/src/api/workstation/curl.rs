use regex::Regex;
use std::process::Command;

pub fn get_version() -> String {
    let output = Command::new("curl")
        .arg("--version")
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8_lossy(&output.stdout);
    let version_regex = Regex::new(r"Curl version (\d+\.\d+\.\d+)").unwrap();
    if let Some(caps) = version_regex.captures(&output) {
        if let Some(version) = caps.get(1) {
            return version.as_str().to_string();
        }
    }
    // Failed to parse version:
    "".to_string()
}
