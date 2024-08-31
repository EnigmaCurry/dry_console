use dry_console_dto::workstation::{Distribution, LinuxRelease, OSType, Platform};
use os_release::OsRelease;
use regex::Regex;
use std::path::Path;

pub fn detect_toolbox() -> bool {
    let toolboxenv = Path::new("/run/.toolboxenv");
    let distroboxenv = Path::new("/run/.distroboxenv");

    toolboxenv.exists() || distroboxenv.exists()
}

pub fn detect_platform() -> Platform {
    if cfg!(target_os = "linux") {
        #[allow(unused_assignments)]
        let mut os_type = OSType::Linux;
        let mut distribution = Distribution::Unsupported;
        let mut version = String::new();
        let mut release = LinuxRelease::default();

        if let Ok(version_info) = std::fs::read_to_string("/proc/version") {
            // Detect if native Linux or inside WSL2
            if version_info.contains("Microsoft") || version_info.contains("WSL2") {
                os_type = OSType::WSL2;
            } else {
                os_type = OSType::Linux;
            }
            // Detect linux version
            let re = Regex::new(r"Linux version (\d+\.\d+\.\d+)").unwrap();
            if let Some(caps) = re.captures(&version_info) {
                version = caps.get(1).map_or("", |m| m.as_str()).to_string();
            } else {
                version = "".to_string();
            }
            // Detect distro
            match OsRelease::new() {
                Ok(r) => {
                    distribution = match r.name.as_str() {
                        "Fedora Linux" => Distribution::Fedora,
                        _ => Distribution::Unsupported,
                    };
                    match distribution {
                        Distribution::Unsupported => release = LinuxRelease::default(),
                        _ => {
                            release = LinuxRelease {
                                name: r.name,
                                version: r.version_id,
                                variant: r.extra.get("VARIANT").unwrap_or(&"".to_string()).clone(),
                                variant_id: r
                                    .extra
                                    .get("VARIANT_ID")
                                    .unwrap_or(&"".to_string())
                                    .clone(),
                            }
                        }
                    }
                }
                Err(_e) => distribution = Distribution::Unsupported,
            };
        } else {
            os_type = OSType::Unknown;
        }

        Platform {
            os_type,
            version,
            distribution,
            release,
        }
    } else if cfg!(target_os = "macos") {
        Platform {
            os_type: OSType::MacOS,
            version: String::new(),
            distribution: Distribution::Unsupported,
            release: LinuxRelease::default(),
        }
    } else {
        Platform {
            os_type: OSType::Unknown,
            version: String::new(),
            distribution: Distribution::Unsupported,
            release: LinuxRelease::default(),
        }
    }
}
