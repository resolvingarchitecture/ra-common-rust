//! A fixed 32-byte identifier. Ports `ra.common.UniqueId`.

use base64::Engine;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{RaError, Result};

/// Number of bytes in a [`UniqueId`].
pub const LENGTH: usize = 32;

/// A 32-byte identifier, rendered as standard padded base64 (44 chars).
///
/// Ordering is unsigned big-endian, which for equal-length arrays is just
/// lexicographic byte ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UniqueId {
    bytes: [u8; LENGTH],
}

impl UniqueId {
    /// A new random id (non-cryptographic, matching the Java `Random` source).
    pub fn random() -> Self {
        let mut bytes = [0u8; LENGTH];
        for b in &mut bytes {
            *b = rand::random();
        }
        UniqueId { bytes }
    }

    /// Copy 32 bytes starting at `offset`.
    ///
    /// # Errors
    /// [`RaError::Invalid`] if `src` has fewer than `offset + 32` bytes.
    pub fn from_slice(src: &[u8], offset: usize) -> Result<Self> {
        let end = offset
            .checked_add(LENGTH)
            .ok_or_else(|| RaError::invalid("offset overflow"))?;
        if src.len() < end {
            return Err(RaError::invalid("not enough bytes for UniqueId"));
        }
        let mut bytes = [0u8; LENGTH];
        bytes.copy_from_slice(&src[offset..end]);
        Ok(UniqueId { bytes })
    }

    /// Build from exactly 32 bytes.
    pub fn from_bytes(bytes: [u8; LENGTH]) -> Self {
        UniqueId { bytes }
    }

    /// Parse a base64-encoded id.
    ///
    /// # Errors
    /// [`RaError::Decode`] if the string is not valid base64 or not 32 bytes.
    pub fn from_base64(s: &str) -> Result<Self> {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(s)
            .map_err(RaError::decode)?;
        let bytes: [u8; LENGTH] = raw
            .try_into()
            .map_err(|_| RaError::decode("UniqueId must be 32 bytes"))?;
        Ok(UniqueId { bytes })
    }

    /// The raw bytes.
    pub fn as_bytes(&self) -> &[u8; LENGTH] {
        &self.bytes
    }

    /// Standard padded base64 encoding.
    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(self.bytes)
    }
}

impl std::fmt::Display for UniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base64())
    }
}

impl Default for UniqueId {
    fn default() -> Self {
        Self::random()
    }
}

impl Serialize for UniqueId {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_base64())
    }
}

impl<'de> Deserialize<'de> for UniqueId {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        UniqueId::from_base64(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trip() {
        let id = UniqueId::random();
        let s = id.to_base64();
        assert_eq!(s.len(), 44);
        assert_eq!(UniqueId::from_base64(&s).unwrap(), id);
    }

    #[test]
    fn ordering_is_unsigned() {
        let lo = UniqueId::from_bytes([0x00; LENGTH]);
        let hi = UniqueId::from_bytes([0xFF; LENGTH]);
        assert!(lo < hi);
    }

    #[test]
    fn from_slice_offset() {
        let mut buf = vec![9u8; 40];
        buf[8..40].copy_from_slice(&[7u8; 32]);
        let id = UniqueId::from_slice(&buf, 8).unwrap();
        assert_eq!(id.as_bytes(), &[7u8; 32]);
        assert!(UniqueId::from_slice(&buf, 9).is_err());
    }
}
