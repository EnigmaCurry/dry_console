use dirs::home_dir;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use uzers::os::unix::UserExt;
use uzers::{get_current_gid, get_current_uid, get_user_by_name};

use tracing::debug;

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if path == "~" {
            // If the path is just "~", expand it to the current user's home directory
            home_dir().unwrap_or_else(|| PathBuf::from("/"))
        } else if let Some(stripped) = path.strip_prefix("~/") {
            // Handle "~/" to the current user's home directory
            if let Some(home) = home_dir() {
                home.join(&stripped)
            } else {
                PathBuf::from(path)
            }
        } else {
            // Handle "~username/..."
            let mut parts = path.splitn(2, '/');
            let user_part = parts.next().unwrap().trim_start_matches('~');
            let rest_of_path = parts.next().unwrap_or("");

            if let Some(user) = get_user_by_name(user_part) {
                PathBuf::from(user.home_dir()).join(rest_of_path)
            } else {
                // If the user doesn't exist, return the path as-is
                PathBuf::from(path)
            }
        }
    } else {
        PathBuf::from(path)
    }
}

pub fn could_create_path(path: &Path) -> io::Result<()> {
    if path.exists() {
        //debug!("already exists");
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "The specified path already exists",
        ));
    }

    if let Some(nearest_parent) = find_nearest_existing_parent(path) {
        if directory_is_writable_by_user(&nearest_parent) {
            Ok(())
        } else {
            //debug!("no write permissions");
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "No write permission on the nearest existing parent directory",
            ))
        }
    } else {
        debug!("no parent found");
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No existing parent directory found",
        ))
    }
}

pub fn path_is_git_repo_root(path: Option<String>) -> bool {
    use std::path::Path;
    use std::process::Command;

    match path {
        None => false,
        Some(path) => {
            let path = Path::new(&path);
            if path.exists() && path.is_dir() {
                let git_dir = path.join(".git");
                if git_dir.exists() && git_dir.is_dir() {
                    return Command::new("git")
                        .arg("-C")
                        .arg(path)
                        .arg("rev-parse")
                        .arg("--is-inside-work-tree")
                        .output()
                        .map(|output| output.status.success())
                        .unwrap_or(false);
                }
            }
            false
        }
    }
}

pub fn find_nearest_existing_parent(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        return None;
    }
    let mut current_path = path.to_path_buf();
    while !current_path.exists() {
        if let Some(parent) = current_path.parent() {
            current_path = parent.to_path_buf();
        } else {
            return None;
        }
    }
    Some(current_path)
}

pub fn directory_is_writable_by_user(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    // Retrieve metadata for the directory
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions().mode(); // Get Unix file mode
        let dir_uid = metadata.uid(); // Get directory owner's UID
        let dir_gid = metadata.gid(); // Get directory owner's GID

        // Get the current user UID and GID using the uzers crate
        let current_uid = get_current_uid();
        let current_gid = get_current_gid();

        // Owner permissions: check if current user is the owner and has write permissions
        if dir_uid == current_uid && (permissions & 0o200 != 0) {
            return true;
        }

        // Group permissions: check if the user's group matches and the group has write permissions
        if dir_gid == current_gid && (permissions & 0o020 != 0) {
            return true;
        }

        // Others permissions: check if others have write access
        if permissions & 0o002 != 0 {
            return true;
        }
    }

    false
}
