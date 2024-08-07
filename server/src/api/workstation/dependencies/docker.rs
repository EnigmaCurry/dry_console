use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "docker --version",
        r"Docker version (\d+\.\d+\.\d+)",
        OutputStream::Stdout,
    )
}
