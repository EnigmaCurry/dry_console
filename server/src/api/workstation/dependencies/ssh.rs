use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version("ssh -V", "OpenSSH_([^ ]*),", OutputStream::Stderr)
}
