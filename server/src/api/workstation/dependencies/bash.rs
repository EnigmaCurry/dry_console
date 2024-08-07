use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "bash --version",
        r"GNU bash, version ([^ ()]+)",
        OutputStream::Stdout,
    )
}
