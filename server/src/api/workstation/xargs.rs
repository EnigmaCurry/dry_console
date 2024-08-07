use crate::api::workstation::find_version;
use crate::api::workstation::OutputStream;

pub fn get_version() -> String {
    find_version(
        "xargs --version",
        r"xargs \(GNU findutils\) ([^\n ]+)",
        OutputStream::Stdout,
    )
}
