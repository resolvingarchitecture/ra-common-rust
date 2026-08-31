//! Ports `ra.common.identity.DID` (Decentralized IDentification).

use serde::{Deserialize, Serialize};

use crate::crypto::{Hash, HashAlgorithm};
use crate::identity::{PiiClearable, PublicKey};

/// Activation state of a [`Did`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DidStatus {
    Inactive,
    Active,
    Suspended,
    Private,
}

/// What a [`Did`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DidType {
    Contact,
    Identity,
    Node,
}

/// A decentralized identity: a username, an optional passphrase (+ its hash),
/// and a [`PublicKey`].
///
/// Deliberately does not follow the W3C DID spec — RA models each key as its own
/// identity rather than grouping keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Did {
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passphrase2: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passphrase_hash: Option<Hash>,
    pub passphrase_hash_algorithm: HashAlgorithm,
    pub description: String,
    pub status: DidStatus,
    pub did_type: DidType,
    pub verified: bool,
    pub authenticated: bool,
    pub public_key: PublicKey,
}

impl Default for Did {
    fn default() -> Self {
        Did {
            username: "Anon".to_string(),
            passphrase: None,
            passphrase2: None,
            passphrase_hash: None,
            passphrase_hash_algorithm: HashAlgorithm::Pbkdf2HmacSha1,
            description: String::new(),
            status: DidStatus::Inactive,
            did_type: DidType::Identity,
            verified: false,
            authenticated: false,
            public_key: PublicKey::default(),
        }
    }
}

impl Did {
    /// A DID with the given username, all else defaulted.
    pub fn with_username(username: impl Into<String>) -> Self {
        Did {
            username: username.into(),
            ..Default::default()
        }
    }

    /// The passphrase-hash algorithm actually recorded on the stored hash, if
    /// any, else the configured default. (The Java getter dereferenced a
    /// possibly-null hash and could panic; this returns an `Option`-free value
    /// by falling back to the field.)
    pub fn passphrase_hash_algorithm(&self) -> HashAlgorithm {
        self.passphrase_hash
            .as_ref()
            .map(|h| h.algorithm)
            .unwrap_or(self.passphrase_hash_algorithm)
    }
}

impl PiiClearable for Did {
    fn clear_sensitive(&mut self) {
        self.username = String::new();
        self.passphrase = None;
        self.passphrase2 = None;
        self.description = String::new();
        self.status = DidStatus::Private;
        self.verified = false;
        self.authenticated = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_java() {
        let d = Did::default();
        assert_eq!(d.username, "Anon");
        assert_eq!(d.status, DidStatus::Inactive);
        assert_eq!(d.did_type, DidType::Identity);
        assert_eq!(d.passphrase_hash_algorithm, HashAlgorithm::Pbkdf2HmacSha1);
    }

    #[test]
    fn clear_sensitive_scrubs() {
        let mut d = Did::with_username("alice");
        d.passphrase = Some("secret".into());
        d.authenticated = true;
        d.clear_sensitive();
        assert!(d.username.is_empty());
        assert!(d.passphrase.is_none());
        assert_eq!(d.status, DidStatus::Private);
        assert!(!d.authenticated);
    }

    #[test]
    fn json_round_trip() {
        let mut d = Did::with_username("bob");
        d.public_key = PublicKey::from_address("addr");
        d.passphrase_hash = Some(Hash::new("deadbeef", HashAlgorithm::Sha256));
        let json = serde_json::to_string(&d).unwrap();
        let back: Did = serde_json::from_str(&json).unwrap();
        assert_eq!(back.username, "bob");
        assert_eq!(back.public_key.address.as_deref(), Some("addr"));
        assert_eq!(back.passphrase_hash.unwrap().hash, "deadbeef");
    }
}
