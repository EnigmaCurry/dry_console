use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "xargs --version",
        r"xargs \(GNU findutils\) ([^\n ]+)",
        OutputStream::Stdout,
    )
}
