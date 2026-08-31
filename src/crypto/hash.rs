//! A hash value paired with the algorithm that produced it.
//!
//! Ports `ra.common.crypto.Hash` and its nested `Algorithm` enum.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::RaError;

/// Digest / key-derivation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha512,
    /// PBKDF2 with HMAC-SHA1 (used for passphrase hashing).
    Pbkdf2HmacSha1,
}

impl HashAlgorithm {
    /// The JCA-style name (`"SHA-256"`, `"PBKDF2WithHmacSHA1"`, ...), matching
    /// the Java `getName()`.
    pub fn as_str(&self) -> &'static str {
        match self {
            HashAlgorithm::Sha1 => "SHA-1",
            HashAlgorithm::Sha256 => "SHA-256",
            HashAlgorithm::Sha512 => "SHA-512",
            HashAlgorithm::Pbkdf2HmacSha1 => "PBKDF2WithHmacSHA1",
        }
    }
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HashAlgorithm {
    type Err = RaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SHA-1" | "SHA1" | "Sha1" => Ok(HashAlgorithm::Sha1),
            "SHA-256" | "SHA256" | "Sha256" => Ok(HashAlgorithm::Sha256),
            "SHA-512" | "SHA512" | "Sha512" => Ok(HashAlgorithm::Sha512),
            "PBKDF2WithHmacSHA1" | "Pbkdf2HmacSha1" => Ok(HashAlgorithm::Pbkdf2HmacSha1),
            other => Err(RaError::invalid(format!("unknown hash algorithm: {other}"))),
        }
    }
}

/// A hash string together with the algorithm used. Equality is on the hash
/// string alone, matching the Java `equals`/`hashCode`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash {
    /// The encoded digest (hex, base64 or the composite password-hash format,
    /// depending on how it was produced).
    pub hash: String,
    /// The algorithm that produced [`Hash::hash`].
    pub algorithm: HashAlgorithm,
}

impl Hash {
    /// Pair a hash string with its algorithm.
    pub fn new(hash: impl Into<String>, algorithm: HashAlgorithm) -> Self {
        Hash {
            hash: hash.into(),
            algorithm,
        }
    }
}

impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Hash {}

impl std::hash::Hash for Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_is_on_string() {
        let a = Hash::new("abc", HashAlgorithm::Sha256);
        let b = Hash::new("abc", HashAlgorithm::Sha1);
        assert_eq!(a, b);
    }

    #[test]
    fn algorithm_names() {
        assert_eq!(HashAlgorithm::Sha256.as_str(), "SHA-256");
        assert_eq!(
            "PBKDF2WithHmacSHA1".parse::<HashAlgorithm>().unwrap(),
            HashAlgorithm::Pbkdf2HmacSha1
        );
    }
}
