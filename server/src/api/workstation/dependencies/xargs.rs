use dry_console_dto::workstation::{Distribution, OSType, Platform};

use crate::api::workstation::{
    dependencies::{find_version, OutputStream},
    WorkstationError, WorkstationPackage, WorkstationPackageManager,
};

pub fn get_version() -> String {
    find_version(
        "xargs --version",
        r"xargs \(GNU findutils\) ([^\n ]+)",
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
                    "findutils",
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
