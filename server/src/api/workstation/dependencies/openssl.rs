use crate::api::workstation::dependencies::{find_version, OutputStream};

pub fn get_version() -> String {
    find_version(
        "openssl --version",
        r"OpenSSL (\d+\.\d+\.\d+)",
        OutputStream::Stdout,
    )
}
