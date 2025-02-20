// rustc version 1.77.2
// tokio version 1.38.0, features: rt-multi-thread, net, io-util

pub use dirs::*;
use libp2p::identity;
pub use std::env;
pub use std::error::Error;
pub use std::hash::RandomState;
pub use std::hash::{BuildHasher, Hasher};
pub use std::time::{SystemTime, UNIX_EPOCH};
pub use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

pub const CUSTOM_PORT: u16 = 6102;

pub fn prepend<T>(v: Vec<T>, s: &[T]) -> Vec<T>
where
    T: Clone,
{
    let mut tmp: Vec<_> = s.to_owned();
    tmp.extend(v);
    tmp
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn random_numbers() -> impl Iterator<Item = u32> {
    let mut random = 92u32;
    std::iter::repeat_with(move || {
        //std::iter::repeat_with(|| {
        random ^= &random << 13;
        random ^= &random >> 17;
        random ^= &random << 5;
        random ^ nanos().unwrap() ^ millis().unwrap()
    })
    .take(10)
}

pub fn random_seed() -> u64 {
    RandomState::new().build_hasher().finish()
}

pub fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}

pub fn millis() -> Result<u32, Box<dyn Error>> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .subsec_millis();
    Ok(millis)
}
pub fn nanos() -> Result<u32, Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
    Ok(nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
