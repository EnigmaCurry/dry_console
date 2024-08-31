use crate::api::workstation::{
    dependencies::{find_version, OutputStream},
    WorkstationError, WorkstationPackage, WorkstationPackageManager,
};
use dry_console_dto::workstation::{Distribution, OSType, Platform};

pub fn get_version() -> String {
    find_version(
        "git --version",
        r"git version (\d+\.\d+\.\d+)",
        OutputStream::Stdout,
    )
}

pub fn get_packages(platform: Platform) -> Result<Vec<WorkstationPackage>, WorkstationError> {
    let mut packages = Vec::<WorkstationPackage>::new();
    match platform.os_type {
        OSType::Linux => {
            match platform.distribution {
                Distribution::Fedora => packages.push(WorkstationPackage::new(
                    WorkstationPackageManager::Dnf,
                    "git",
                )),
                Distribution::Arch => return Err(WorkstationError::UnsupportedDistribution),
                Distribution::Debian => return Err(WorkstationError::UnsupportedDistribution),
                Distribution::Ubuntu => return Err(WorkstationError::UnsupportedDistribution),
                Distribution::Unsupported => return Err(WorkstationError::UnsupportedDistribution),
            };
            Ok(packages)
        }
        OSType::MacOS => Err(WorkstationError::UnsupportedPlatform),
        OSType::WSL2 => Err(WorkstationError::UnsupportedPlatform),
        OSType::Unknown => Err(WorkstationError::UnsupportedPlatform),
    }
}
