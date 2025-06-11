use crate::get_time_nanoseconds;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

pub trait ChaCha20RngExt {
    fn from_nano_time() -> Self;
}

impl ChaCha20RngExt for ChaCha20Rng {
    fn from_nano_time() -> Self {
        let nanos = get_time_nanoseconds();
        let mut seed = [0u8; 32];

        seed[..16].copy_from_slice(&nanos.to_le_bytes());

        ChaCha20Rng::from_seed(seed)
    }
}