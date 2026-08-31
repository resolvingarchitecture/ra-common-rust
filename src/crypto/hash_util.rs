//! Digests, fingerprints and passphrase hashing. Ports `ra.common.HashUtil`.
//!
//! Formats produced here (`b64(hash)_b64(salt)` for salted digests,
//! `iterations_b64(salt)_b64(hash)` for PBKDF2) mirror the Java layout but are
//! **this crate's own format** — they are not required to interoperate with the
//! Java library.

use base64::Engine;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use subtle::ConstantTimeEq;

use crate::crypto::hash::HashAlgorithm;
use crate::error::{RaError, Result};

const PBKDF2_ITERATIONS: u32 = 1000;
const PBKDF2_KEY_LEN: usize = 64;
const SALT_LEN: usize = 16;

fn b64() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// 16 cryptographically-secure random salt bytes.
pub fn salt() -> [u8; SALT_LEN] {
    let mut s = [0u8; SALT_LEN];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut s);
    s
}

/// Raw digest of `data` with `algorithm`.
///
/// # Errors
/// [`RaError::Crypto`] if called with [`HashAlgorithm::Pbkdf2HmacSha1`] (which is
/// a key-derivation function, not a plain digest).
pub fn digest(data: &[u8], algorithm: HashAlgorithm) -> Result<Vec<u8>> {
    Ok(match algorithm {
        HashAlgorithm::Sha1 => Sha1::digest(data).to_vec(),
        HashAlgorithm::Sha256 => Sha256::digest(data).to_vec(),
        HashAlgorithm::Sha512 => Sha512::digest(data).to_vec(),
        HashAlgorithm::Pbkdf2HmacSha1 => {
            return Err(RaError::crypto("PBKDF2 is not a plain digest"))
        }
    })
}

/// Uppercase hex of `bytes`, grouped into blocks of four characters separated by
/// `:` (matching the Java `toHex`).
pub fn to_hex(bytes: &[u8]) -> String {
    let hex = hex::encode_upper(bytes);
    hex.as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(":")
}

/// Inverse of [`to_hex`]; `:` separators are ignored.
///
/// # Errors
/// [`RaError::Decode`] if the remaining characters are not valid hex.
pub fn from_hex(s: &str) -> Result<Vec<u8>> {
    let clean: String = s.chars().filter(|c| *c != ':').collect();
    hex::decode(clean).map_err(RaError::decode)
}

/// Fingerprint: the digest of `data`, hex-encoded via [`to_hex`].
///
/// # Errors
/// Propagates [`digest`] errors.
pub fn generate_fingerprint(data: &[u8], algorithm: HashAlgorithm) -> Result<String> {
    Ok(to_hex(&digest(data, algorithm)?))
}

/// Hash `content`. For [`HashAlgorithm::Pbkdf2HmacSha1`] this produces a
/// passphrase hash; otherwise it salts and digests, returning
/// `b64(digest(salt ++ content))_b64(salt)`.
pub fn generate_hash(content: &[u8], algorithm: HashAlgorithm) -> Result<String> {
    if algorithm == HashAlgorithm::Pbkdf2HmacSha1 {
        return Ok(generate_password_hash(std::str::from_utf8(content).map_err(RaError::invalid)?));
    }
    let s = salt();
    let mut buf = Vec::with_capacity(s.len() + content.len());
    buf.extend_from_slice(&s);
    buf.extend_from_slice(content);
    let h = digest(&buf, algorithm)?;
    Ok(format!("{}_{}", b64().encode(h), b64().encode(s)))
}

/// Verify `content` against a string produced by [`generate_hash`].
pub fn verify_hash(content: &[u8], hash_to_verify: &str, algorithm: HashAlgorithm) -> Result<bool> {
    if algorithm == HashAlgorithm::Pbkdf2HmacSha1 {
        return Ok(verify_password_hash(
            std::str::from_utf8(content).map_err(RaError::invalid)?,
            hash_to_verify,
        ));
    }
    let (h_b64, s_b64) = hash_to_verify
        .split_once('_')
        .ok_or_else(|| RaError::invalid("malformed hash"))?;
    let expected = b64().decode(h_b64).map_err(RaError::decode)?;
    let s = b64().decode(s_b64).map_err(RaError::decode)?;
    let mut buf = Vec::with_capacity(s.len() + content.len());
    buf.extend_from_slice(&s);
    buf.extend_from_slice(content);
    let actual = digest(&buf, algorithm)?;
    Ok(bool::from(actual.ct_eq(&expected)))
}

/// PBKDF2-HMAC-SHA1 passphrase hash: `iterations_b64(salt)_b64(derived)`,
/// 1000 iterations, 64-byte derived key.
pub fn generate_password_hash(password: &str) -> String {
    generate_password_hash_with_salt(password, &salt())
}

fn generate_password_hash_with_salt(password: &str, s: &[u8]) -> String {
    let mut out = [0u8; PBKDF2_KEY_LEN];
    pbkdf2::pbkdf2_hmac::<Sha1>(password.as_bytes(), s, PBKDF2_ITERATIONS, &mut out);
    format!(
        "{}_{}_{}",
        PBKDF2_ITERATIONS,
        b64().encode(s),
        b64().encode(out)
    )
}

/// Verify a passphrase against a string from [`generate_password_hash`].
pub fn verify_password_hash(password: &str, hash_to_verify: &str) -> bool {
    let parts: Vec<&str> = hash_to_verify.split('_').collect();
    if parts.len() != 3 {
        return false;
    }
    let Ok(iterations) = parts[0].parse::<u32>() else {
        return false;
    };
    let (Ok(s), Ok(expected)) = (b64().decode(parts[1]), b64().decode(parts[2])) else {
        return false;
    };
    let mut out = vec![0u8; expected.len()];
    pbkdf2::pbkdf2_hmac::<Sha1>(password.as_bytes(), &s, iterations, &mut out);
    bool::from(out.ct_eq(&expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_grouping() {
        assert_eq!(to_hex(&[0x0a, 0x0b, 0x0c, 0x0d, 0x1e]), "0A0B:0C0D:1E");
        assert_eq!(from_hex("0A0B:0C0D:1E").unwrap(), vec![0x0a, 0x0b, 0x0c, 0x0d, 0x1e]);
    }

    #[test]
    fn salted_hash_round_trip() {
        let h = generate_hash(b"Alice", HashAlgorithm::Sha256).unwrap();
        assert!(verify_hash(b"Alice", &h, HashAlgorithm::Sha256).unwrap());
        assert!(!verify_hash(b"Bob", &h, HashAlgorithm::Sha256).unwrap());
    }

    #[test]
    fn password_hash_round_trip() {
        let h = generate_password_hash("hunter2");
        assert!(h.starts_with("1000_"));
        assert!(verify_password_hash("hunter2", &h));
        assert!(!verify_password_hash("hunter3", &h));
    }

    #[test]
    fn password_hash_via_generate_hash() {
        let h = generate_hash(b"pw", HashAlgorithm::Pbkdf2HmacSha1).unwrap();
        assert!(verify_hash(b"pw", &h, HashAlgorithm::Pbkdf2HmacSha1).unwrap());
    }

    #[test]
    fn fingerprint_is_stable() {
        let a = generate_fingerprint(b"x", HashAlgorithm::Sha1).unwrap();
        let b = generate_fingerprint(b"x", HashAlgorithm::Sha1).unwrap();
        assert_eq!(a, b);
        assert!(a.contains(':'));
    }
}
