use rand::{distributions::Alphanumeric, Rng};

const TOKEN_LENGTH: usize = 32;

pub fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(TOKEN_LENGTH)
        .map(char::from)
        .collect()
}
