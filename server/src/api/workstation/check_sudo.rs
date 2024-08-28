use std::process::Stdio;
use tokio::process::Command;

pub async fn check_sudo() -> bool {
    let command = match Command::new("groups")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(cmd) => cmd,
        Err(_) => return false,
    };

    let output = match command.wait_with_output().await {
        Ok(output) => output,
        Err(_) => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let groups: Vec<&str> = stdout.trim().split_whitespace().collect();

    groups.contains(&"wheel") || groups.contains(&"sudo")
}
