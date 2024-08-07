use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "sed --version",
        r"sed \(GNU sed\) ([^\n ]+)",
        OutputStream::Stdout,
    )
}
