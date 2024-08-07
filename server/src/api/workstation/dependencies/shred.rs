use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "shred --version",
        r"shred \(GNU coreutils\) ([^\n ]+)",
        OutputStream::Stdout,
    )
}
