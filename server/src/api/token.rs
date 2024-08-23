use rand::{distributions::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use ulid::Ulid;

const TOKEN_LENGTH: usize = 32;

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(TOKEN_LENGTH)
        .map(char::from)
        .collect()
}

pub fn generate_deterministic_ulid_from_seed(seed: &str) -> Ulid {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    let result = hasher.finalize();
    // Use the first 16 bytes of the hash to create a ULID
    let bytes = &result[..16];
    Ulid::from_bytes(bytes.try_into().expect("slice with incorrect length"))
}
