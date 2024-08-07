use dry_console_dto::workstation::{Distribution, OSType, Platform, PlatformSupport};
use regex::Regex;
use serde::Serialize;
use utoipa::ToSchema;
pub fn detect_platform() -> Platform {
    if cfg!(target_os = "linux") {
        let mut os_type = OSType::Linux;
        let mut supported = PlatformSupport::Unsupported;
        let mut distribution = Distribution::Unknown;
        let mut version = String::new();

        if let Ok(version_info) = std::fs::read_to_string("/proc/version") {
            if version_info.contains("Microsoft") || version_info.contains("WSL2") {
                os_type = OSType::WSL2;
            } else {
                os_type = OSType::Linux;
            }
            let re = Regex::new(r"Linux version (\d+\.\d+\.\d+)").unwrap();
            if let Some(caps) = re.captures(&version_info) {
                version = caps.get(1).map_or("", |m| m.as_str()).to_string();
            } else {
                version = "".to_string();
            }
        } else {
            os_type = OSType::Unknown;
        }

        Platform {
            os_type,
            version,
            supported,
            distribution,
        }
    } else if cfg!(target_os = "macos") {
        Platform {
            os_type: OSType::MacOS,
            version: String::new(),
            supported: PlatformSupport::Unsupported,
            distribution: Distribution::Unknown,
        }
    } else {
        Platform {
            os_type: OSType::Unknown,
            version: String::new(),
            supported: PlatformSupport::Unsupported,
            distribution: Distribution::Unknown,
        }
    }
}
