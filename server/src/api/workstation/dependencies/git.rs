use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "git --version",
        r"git version (\d+\.\d+\.\d+)",
        OutputStream::Stdout,
    )
}
