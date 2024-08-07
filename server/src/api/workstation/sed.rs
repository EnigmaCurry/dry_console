use crate::api::workstation::find_version;
use crate::api::workstation::OutputStream;

pub fn get_version() -> String {
    find_version(
        "sed --version",
        r"sed \(GNU sed\) ([^\n ]+)",
        OutputStream::Stdout,
    )
}
