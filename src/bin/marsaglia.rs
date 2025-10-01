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
