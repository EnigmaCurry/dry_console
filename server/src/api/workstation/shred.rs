use crate::api::workstation::find_version;
use crate::api::workstation::OutputStream;

pub fn get_version() -> String {
    find_version(
        "shred --version",
        r"shred \(GNU coreutils\) ([^\n ]+)",
        OutputStream::Stdout,
    )
}
