use serde::{Deserialize, Serialize};
use strum::Display;
use utoipa::ToSchema;

#[derive(Default, Serialize, Deserialize, ToSchema, Clone)]
pub struct WorkstationUser {
    pub uid: u32,
    pub name: String,
}

#[derive(Default, Serialize, Deserialize, ToSchema, Clone)]
pub struct WorkstationState {
    /// Hostname of the workstation.
    pub hostname: String,
    pub user: WorkstationUser,
    pub platform: Platform,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum WorkstationPackageManager {
    Dnf,
    Dpkg,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct WorkstationPackage {
    pub package_manager: WorkstationPackageManager,
    pub package_name: String,
}

impl WorkstationPackage {
    pub fn new(package_manager: WorkstationPackageManager, package_name: &str) -> Self {
        WorkstationPackage {
            package_manager,
            package_name: package_name.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct WorkstationDependencyInfo {
    pub name: String,
    pub version: String,
    pub packages: Vec<WorkstationPackage>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Display)]
pub enum OSType {
    Linux,
    MacOS,
    WSL2,
    Unknown,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Display, PartialEq)]
pub enum Distribution {
    Fedora,
    Arch,
    Debian,
    Ubuntu,
    Unsupported,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct LinuxRelease {
    pub name: String,
    pub version: String,
    pub variant: String,
    pub variant_id: String,
}
impl Default for LinuxRelease {
    fn default() -> Self {
        Self {
            name: "Unsupported".to_string(),
            version: "".to_string(),
            variant: "".to_string(),
            variant_id: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Platform {
    pub os_type: OSType,
    pub version: String,
    pub distribution: Distribution,
    pub release: LinuxRelease,
}
impl Default for Platform {
    fn default() -> Self {
        Self {
            os_type: OSType::Unknown,
            version: String::new(),
            distribution: Distribution::Unsupported,
            release: LinuxRelease::default(),
        }
    }
}
