use log::debug;
use std::error::Error;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

pub const CUSTOM_PORT: usize = 8000;

pub fn prepend<T>(v: Vec<T>, s: &[T]) -> Vec<T>
where
    T: Clone,
{
    let mut tmp: Vec<_> = s.to_owned();
    tmp.extend(v);
    tmp
}

// Pseudorandom number generator from the "Xorshift RNGs" paper by George Marsaglia.
//
// https://github.com/rust-lang/rust/blob/1.55.0/library/core/src/slice/sort.rs#L559-L573
pub fn random_numbers() -> impl Iterator<Item = u32> {
    let mut random = 92u32;
    std::iter::repeat_with(move || {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        random
    })
}

pub fn random_seed() -> u64 {
    RandomState::new().build_hasher().finish()
}

pub fn nanos() -> Result<u32, Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.subsec_nanos();
    debug!("nanos(): {nanos}");
    Ok(nanos)
}

pub fn func_add(left: u64, right: u64) -> u64 {
    left + right
}
pub fn func_xor(left: u64, right: u64) -> u64 {
    left ^ right
}

pub struct Xorshift128 {
    // The state consists of four 32-bit unsigned integers (u32)
    // representing the 128-bit state vector.
    x: u32,
    y: u32,
    z: u32,
    w: u32,
}

impl Xorshift128 {
    /// Creates a new Xorshift128 generator with initial seeds.
    ///
    /// The seeds must not all be zero, as the all-zero state is a fixed point.
    pub fn new(seed_x: u32, seed_y: u32, seed_z: u32, seed_w: u32) -> Self {
        // Simple check to ensure not all seeds are zero
        if seed_x == 0 && seed_y == 0 && seed_z == 0 && seed_w == 0 {
            // In a real library, you'd likely use a default non-zero seed or panic.
            // For this example, we'll use the nanos() instead of default seeds from the paper.
            Xorshift128 {
                x: nanos().expect(""), // 123456789
                y: nanos().expect(""), // 362436069
                z: nanos().expect(""), // 521288629
                w: nanos().expect(""), // 88675123
            }
        } else {
            Xorshift128 {
                x: seed_x,
                y: seed_y,
                z: seed_z,
                w: seed_w,
            }
        }
    }

    /// Generates the next pseudo-random 32-bit unsigned integer.
    pub fn next(&mut self) -> u32 {
        // 1. Calculate the temporary value 't' using the current 'x'
        let t = self.x ^ (self.x << 11);

        // 2. State promotion (shift the state variables): x = y, y = z, z = w
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;

        // 3. Calculate the new 'w' and the final result:
        //    w = (w ^ (w >> 19)) ^ (t ^ (t >> 8))
        self.w = (self.w ^ (self.w >> 19)) ^ (t ^ (t >> 8));

        // 4. Return the new 'w' as the random number
        self.w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let result = func_add(2, 2);
        assert_eq!(result, 4);
    }
    #[test]
    fn xor() {
        let result = func_xor(2, 2);
        assert_eq!(result, 0);
    }
}
