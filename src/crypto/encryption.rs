//! Ports `ra.common.crypto.EncryptionAlgorithm`.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::RaError;

/// Symmetric encryption algorithm label. (These are just identifiers carried in
/// [`crate::content::Content`] metadata; this crate does not encrypt.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// CAST-128 / CAST5.
    Cast5,
    /// AES-256.
    Aes256,
    /// "AES-512" (as labelled by the Java original; not a real cipher).
    Aes512,
}

impl EncryptionAlgorithm {
    /// The wire/display name, matching the Java `getName()`.
    pub fn as_str(&self) -> &'static str {
        match self {
            EncryptionAlgorithm::Cast5 => "CAST-5",
            EncryptionAlgorithm::Aes256 => "AES-256",
            EncryptionAlgorithm::Aes512 => "AES-512",
        }
    }
}

impl std::fmt::Display for EncryptionAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for EncryptionAlgorithm {
    type Err = RaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAST-5" | "Cast5" => Ok(EncryptionAlgorithm::Cast5),
            "AES-256" | "Aes256" => Ok(EncryptionAlgorithm::Aes256),
            "AES-512" | "Aes512" => Ok(EncryptionAlgorithm::Aes512),
            other => Err(RaError::invalid(format!("unknown encryption algorithm: {other}"))),
        }
    }
}
