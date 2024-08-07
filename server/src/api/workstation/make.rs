use crate::api::workstation::find_version;
use crate::api::workstation::OutputStream;

pub fn get_version() -> String {
    find_version(
        "make --version",
        r"GNU Make (\d+\.\d+\.\d+)",
        OutputStream::Stdout,
    )
}
