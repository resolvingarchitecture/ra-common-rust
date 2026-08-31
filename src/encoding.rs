//! Base32 and Base58 string codecs.
//!
//! Ports `ra.common.Base32` and `ra.common.Base58`. The Java versions were
//! `BigInteger` base-conversions with bespoke leading-zero handling; since this
//! port is not wire-compatible we use standard implementations:
//!
//! - **base32**: RFC 4648, uppercase `A-Z2-7`, no padding ([`data_encoding::BASE32_NOPAD`]).
//! - **base58**: the Bitcoin alphabet ([`bs58`]).

use crate::error::{RaError, Result};

/// Encode bytes as unpadded RFC 4648 base32 (`A-Z2-7`).
pub fn base32_encode(input: &[u8]) -> String {
    data_encoding::BASE32_NOPAD.encode(input)
}

/// Decode an unpadded RFC 4648 base32 string.
///
/// # Errors
/// [`RaError::Decode`] on any invalid character or length.
pub fn base32_decode(input: &str) -> Result<Vec<u8>> {
    data_encoding::BASE32_NOPAD
        .decode(input.as_bytes())
        .map_err(RaError::decode)
}

/// Encode bytes as base58 (Bitcoin alphabet).
pub fn base58_encode(input: &[u8]) -> String {
    bs58::encode(input).into_string()
}

/// Decode a base58 (Bitcoin alphabet) string.
///
/// # Errors
/// [`RaError::Decode`] on any invalid character.
pub fn base58_decode(input: &str) -> Result<Vec<u8>> {
    bs58::decode(input).into_vec().map_err(RaError::decode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base32_round_trip() {
        let data = b"resolving architecture";
        let enc = base32_encode(data);
        assert!(enc.bytes().all(|b| b.is_ascii_uppercase() || (b'2'..=b'7').contains(&b)));
        assert_eq!(base32_decode(&enc).unwrap(), data);
    }

    #[test]
    fn base58_known_vector() {
        // "Hello World!" -> base58 (Bitcoin alphabet)
        assert_eq!(base58_encode(b"Hello World!"), "2NEpo7TZRRrLZSi2U");
        assert_eq!(base58_decode("2NEpo7TZRRrLZSi2U").unwrap(), b"Hello World!");
    }

    #[test]
    fn base58_rejects_bad_char() {
        assert!(base58_decode("0OIl").is_err());
    }
}
