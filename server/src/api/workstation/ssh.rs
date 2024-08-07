use crate::api::workstation::find_version;
use crate::api::workstation::OutputStream;

pub fn get_version() -> String {
    find_version("ssh -V", "OpenSSH_([^ ]*),", OutputStream::Stderr)
}
