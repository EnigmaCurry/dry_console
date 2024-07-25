use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn generate_secure_passphrase(length: usize) -> String {
    let rng = thread_rng();
    let passphrase: Vec<u8> = rng.sample_iter(&Alphanumeric).take(length).collect();
    String::from_utf8_lossy(&passphrase).to_string()
}
