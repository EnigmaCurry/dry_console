use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version("jq --version", r"jq-(\d+\.\d+\.\d+)", OutputStream::Stdout)
}
