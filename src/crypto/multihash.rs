//! A simplified multihash: a one-byte type code, a one-byte length, then the
//! digest. Ports `ra.common.crypto.Multihash`.
//!
//! Note this is the Java library's fixed-2-byte-header variant, **not** the full
//! varint multihash spec.

use serde::{Deserialize, Serialize};

use crate::encoding::{base58_decode, base58_encode};
use crate::error::{RaError, Result};

/// Known multihash digest types with their code and digest length in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MultihashType {
    Sha1,
    Sha2_256,
    Sha2_512,
    Sha3,
    Blake2b,
    Blake2s,
}

impl MultihashType {
    /// The multihash type code (`0x11` for sha1, ...).
    pub fn code(&self) -> u8 {
        match self {
            MultihashType::Sha1 => 0x11,
            MultihashType::Sha2_256 => 0x12,
            MultihashType::Sha2_512 => 0x13,
            MultihashType::Sha3 => 0x14,
            MultihashType::Blake2b => 0x40,
            MultihashType::Blake2s => 0x41,
        }
    }

    /// The expected digest length in bytes.
    pub fn length(&self) -> usize {
        match self {
            MultihashType::Sha1 => 20,
            MultihashType::Sha2_256 => 32,
            MultihashType::Sha2_512 => 64,
            MultihashType::Sha3 => 64,
            MultihashType::Blake2b => 64,
            MultihashType::Blake2s => 32,
        }
    }

    /// Look up a type by its code.
    ///
    /// # Errors
    /// [`RaError::Invalid`] for an unknown code.
    pub fn from_code(code: u8) -> Result<Self> {
        Ok(match code {
            0x11 => MultihashType::Sha1,
            0x12 => MultihashType::Sha2_256,
            0x13 => MultihashType::Sha2_512,
            0x14 => MultihashType::Sha3,
            0x40 => MultihashType::Blake2b,
            0x41 => MultihashType::Blake2s,
            other => return Err(RaError::invalid(format!("unknown multihash type: {other:#x}"))),
        })
    }
}

/// A digest tagged with its type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Multihash {
    #[serde(rename = "type")]
    kind: MultihashType,
    hash: Vec<u8>,
}

impl Multihash {
    /// Build from a type and its digest bytes.
    ///
    /// # Errors
    /// [`RaError::Invalid`] if `hash` is longer than 127 bytes or does not match
    /// `kind`'s expected length.
    pub fn new(kind: MultihashType, hash: Vec<u8>) -> Result<Self> {
        if hash.len() > 127 {
            return Err(RaError::invalid(format!("unsupported hash size: {}", hash.len())));
        }
        if hash.len() != kind.length() {
            return Err(RaError::invalid(format!(
                "incorrect hash length: {} != {}",
                hash.len(),
                kind.length()
            )));
        }
        Ok(Multihash { kind, hash })
    }

    /// The type.
    pub fn kind(&self) -> MultihashType {
        self.kind
    }

    /// The digest bytes.
    pub fn digest(&self) -> &[u8] {
        &self.hash
    }

    /// `[code, len, ...digest]`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.hash.len() + 2);
        out.push(self.kind.code());
        out.push(self.hash.len() as u8);
        out.extend_from_slice(&self.hash);
        out
    }

    /// Parse the 2-byte-header wire form.
    ///
    /// # Errors
    /// [`RaError::Invalid`] on a short buffer, unknown code or length mismatch.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 2 {
            return Err(RaError::invalid("multihash too short"));
        }
        let kind = MultihashType::from_code(bytes[0])?;
        let len = bytes[1] as usize;
        if bytes.len() < 2 + len {
            return Err(RaError::invalid("multihash truncated"));
        }
        Multihash::new(kind, bytes[2..2 + len].to_vec())
    }

    /// Lowercase, zero-padded hex of [`Multihash::to_bytes`].
    ///
    /// (The Java `toHex` used `%x` and was not zero-padded; this is the fixed
    /// version.)
    pub fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    /// Parse from hex.
    ///
    /// # Errors
    /// [`RaError::Decode`] / [`RaError::Invalid`] on malformed input.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex::decode(s).map_err(RaError::decode)?;
        Multihash::from_bytes(&bytes)
    }

    /// Base58 (Bitcoin alphabet) of [`Multihash::to_bytes`].
    pub fn to_base58(&self) -> String {
        base58_encode(&self.to_bytes())
    }

    /// Parse from base58.
    ///
    /// # Errors
    /// [`RaError::Decode`] / [`RaError::Invalid`] on malformed input.
    pub fn from_base58(s: &str) -> Result<Self> {
        Multihash::from_bytes(&base58_decode(s)?)
    }
}

impl std::fmt::Display for Multihash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base58())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Multihash {
        Multihash::new(MultihashType::Sha2_256, vec![0xAB; 32]).unwrap()
    }

    #[test]
    fn bytes_round_trip() {
        let m = sample();
        let bytes = m.to_bytes();
        assert_eq!(bytes[0], 0x12);
        assert_eq!(bytes[1], 32);
        assert_eq!(Multihash::from_bytes(&bytes).unwrap(), m);
    }

    #[test]
    fn hex_and_base58_round_trip() {
        let m = sample();
        assert_eq!(Multihash::from_hex(&m.to_hex()).unwrap(), m);
        assert_eq!(Multihash::from_base58(&m.to_base58()).unwrap(), m);
    }

    #[test]
    fn length_is_validated() {
        assert!(Multihash::new(MultihashType::Sha1, vec![0; 10]).is_err());
        assert!(MultihashType::from_code(0x99).is_err());
    }
}
