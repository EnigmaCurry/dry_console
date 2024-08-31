use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

#[allow(dead_code)]
pub fn generate_random_string(length: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
