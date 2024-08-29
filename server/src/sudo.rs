use std::io::{self, ErrorKind};
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{self, Duration};
use tracing::{error, info};

/// Acquire sudo, blocking on authentication, but timeout eventually.
pub async fn acquire_sudo(timeout: u64) -> io::Result<()> {
    let command_future = Command::new("sudo")
        .arg("whoami")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait_with_output();

    match time::timeout(Duration::from_secs(timeout), command_future).await {
        Ok(Ok(output)) if output.status.success() => Ok(()),
        Ok(Ok(_)) => Err(io::Error::new(
            ErrorKind::Other,
            "failed to authenticate with sudo",
        )),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(io::Error::new(
            ErrorKind::TimedOut,
            "sudo command timed out",
        )),
    }
}

/// Thread continually keeps the sudo session active by running whoami every 60s.
/// If authentication fails, terminate thread.
///
///   #[tokio::main]
///   async fn main() {
///       tokio::spawn(async {
///           keep_sudo_session_alive().await;
///       });
///   }
pub async fn keep_sudo_session_alive(refresh_interval: u64, sudo_timeout: u64) {
    loop {
        match acquire_sudo(sudo_timeout).await {
            Ok(_) => {
                info!("Root (sudo) authentication successful, will re-authenticate in {refresh_interval}s.");
                time::sleep(Duration::from_secs(refresh_interval)).await;
            }
            Err(e) => {
                error!("Failed to acquire sudo authentication: {:?}", e);
                break;
            }
        }
    }
}
