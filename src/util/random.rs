//! Non-cryptographic random helpers. Ports `ra.common.RandomUtil`.
//!
//! Like the Java original these use the thread-local PRNG, **not** a CSPRNG.
//! For salts and key material use [`crate::crypto`] instead.

use rand::distributions::{Alphanumeric, Distribution, Uniform};
use rand::Rng;

/// A random `i64` across the full range (upper bound exclusive, as in Java).
pub fn next_long() -> i64 {
    rand::thread_rng().gen_range(i64::MIN..i64::MAX)
}

/// A random `i64` in `i64::MIN..upper`.
pub fn next_long_to(upper: i64) -> i64 {
    rand::thread_rng().gen_range(i64::MIN..upper)
}

/// A random `i64` in `lower..upper`.
pub fn next_long_in(lower: i64, upper: i64) -> i64 {
    rand::thread_rng().gen_range(lower..upper)
}

/// A random `i32` across the full range (upper bound exclusive, as in Java).
pub fn next_int() -> i32 {
    rand::thread_rng().gen_range(i32::MIN..i32::MAX)
}

/// A random `i32` in `i32::MIN..upper`.
pub fn next_int_to(upper: i32) -> i32 {
    rand::thread_rng().gen_range(i32::MIN..upper)
}

/// A random `i32` in `lower..upper`.
pub fn next_int_in(lower: i32, upper: i32) -> i32 {
    rand::thread_rng().gen_range(lower..upper)
}

/// A random string of `length` characters drawn from `[0-9A-Za-z]`.
pub fn random_alphanumeric(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// `count` random bytes from the thread PRNG.
pub fn random_bytes(count: usize) -> Vec<u8> {
    let dist = Uniform::new_inclusive(0u8, 255u8);
    let mut rng = rand::thread_rng();
    (0..count).map(|_| dist.sample(&mut rng)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphanumeric_shape() {
        let s = random_alphanumeric(64);
        assert_eq!(s.len(), 64);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn ranges_hold() {
        for _ in 0..1000 {
            let v = next_int_in(-5, 5);
            assert!((-5..5).contains(&v));
        }
    }
}
