use std::fs;
use std::io;
use std::path::Path;

use tracing::debug;

pub fn could_create_path(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "The specified path already exists",
        ));
    }
    let mut current_path = path.to_path_buf();
    // Traverse upwards until we find an existing directory
    while !current_path.exists() {
        if let Some(parent) = current_path.parent() {
            current_path = parent.to_path_buf();
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "No existing parent directory found",
            ));
        }
    }
    // check for write permission on the found existing directory
    let _dir = fs::OpenOptions::new().write(true).open(current_path)?;
    Ok(())
}

pub fn path_is_git_repo_root(path: Option<String>) -> bool {
    use std::path::Path;
    use std::process::Command;

    match path {
        None => false,
        Some(path) => {
            let path = Path::new(&path);
            debug!("checking pathzz : {:?}", path);
            if path.exists() && path.is_dir() {
                let git_dir = path.join(".git");
                debug!("checking .git dir : {:?}", git_dir);
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
