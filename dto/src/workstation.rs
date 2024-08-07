use serde::Serialize;
use strum::{AsRefStr, EnumIter, EnumProperty, EnumString, IntoEnumIterator};
use utoipa::ToSchema;

#[derive(Default, Serialize, ToSchema)]
pub struct WorkstationUser {
    pub uid: u32,
    pub name: String,
}

#[derive(Default, Serialize, ToSchema)]
pub struct WorkstationState {
    /// Hostname of the workstation.
    pub hostname: String,
    pub user: WorkstationUser,
    pub platform: Platform,
}

#[derive(Serialize)]
pub struct WorkstationDependencyInfo {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, ToSchema)]
pub enum OSType {
    Linux,
    MacOS,
    WSL2,
    Unknown,
}

#[derive(Serialize, ToSchema)]
pub enum PlatformSupport {
    FullySupported,
    PartiallySupported,
    Unsupported,
}

#[derive(Serialize, ToSchema)]
pub enum Distribution {
    Fedora,
    Arch,
    Debian,
    Ubuntu,
    Unknown,
}

#[derive(Serialize, ToSchema)]
pub struct Platform {
    pub os_type: OSType,
    pub version: String,
    pub supported: PlatformSupport,
    pub distribution: Distribution,
}
impl Default for Platform {
    fn default() -> Self {
        Self {
            os_type: OSType::Unknown,
            version: String::new(),
            supported: PlatformSupport::Unsupported,
            distribution: Distribution::Unknown,
        }
    }
}
