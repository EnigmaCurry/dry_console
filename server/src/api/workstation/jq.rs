use crate::api::workstation::find_version;
use crate::api::workstation::OutputStream;

pub fn get_version() -> String {
    find_version("jq --version", r"jq-(\d+\.\d+\.\d+)", OutputStream::Stdout)
}
