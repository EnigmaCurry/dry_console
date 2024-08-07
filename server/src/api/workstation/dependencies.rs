use regex::Regex;
use std::process::Command;

pub mod bash;
pub mod curl;
pub mod docker;
pub mod git;
pub mod htpasswd;
pub mod jq;
pub mod make;
pub mod openssl;
pub mod sed;
pub mod shred;
pub mod ssh;
pub mod xargs;
pub mod xdg_open;

pub enum OutputStream {
    ///Process stdout
    Stdout,
    ///Process stderr
    Stderr,
}

///Find the version of a program by matching its output to regex
pub fn find_version(cmd: &str, regex: &str, stream: OutputStream) -> String {
    if let Ok(parts) = shell_words::split(cmd) {
        if let Some((program, args)) = parts.split_first() {
            if let Ok(output) = Command::new(program).args(args).output() {
                let output = match stream {
                    OutputStream::Stdout => String::from_utf8_lossy(&output.stdout).to_string(),
                    OutputStream::Stderr => String::from_utf8_lossy(&output.stderr).to_string(),
                };

                if let Ok(version_regex) = Regex::new(regex) {
                    if let Some(caps) = version_regex.captures(&output) {
                        if let Some(version) = caps.get(1) {
                            return version.as_str().to_string();
                        }
                    }
                }
            }
        }
    }
    "".to_string()
}
