use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "curl --version",
        r"curl (\d+\.\d+\.\d+)",
        OutputStream::Stdout,
    )
}
