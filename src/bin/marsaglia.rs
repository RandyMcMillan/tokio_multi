use tokio_multi::Xorshift128;

fn main() {
    // 1. Initialize the RNG with arbitrary non-zero seeds
    //let mut rng = Xorshift128::new(10, 20, 30, 40);

    //println!("Generating 10 pseudo-random 32-bit integers:");
    //for i in 0..10 {
    //    let rand_val = rng.next();
    //    println!("{}: {}", i + 1, rand_val);
    //}
    let mut rng = Xorshift128::new(0, 0, 0, 0); // zero case invokes the use of nanos() function

    println!("Generating 10 pseudo-random 32-bit integers from time nanos:");
    for i in 0..10 {
        let rand_val = rng.next();
        println!("{}: {}", i + 1, rand_val);
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    use crate::Xorshift128;
    use std::sync::{LazyLock, Mutex};

    static RNG: LazyLock<Mutex<Xorshift128>> = LazyLock::new(|| {
        let rng_instance = Xorshift128::new(10, 20, 30, 40);
        Mutex::new(rng_instance)
    });

    #[test]
    fn test_static_rng_access() {
        println!("Generating 10 pseudo-random 32-bit integers using static RNG:");
        for i in 0..10 {
            let mut rng_guard = RNG.lock().unwrap();
            let rand_val = rng_guard.next();
            println!("{}: {}", i + 1, rand_val);
        }
        assert!(true);
    }
}
