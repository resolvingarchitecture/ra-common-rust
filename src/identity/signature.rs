//! Ports `ra.common.identity.Signature`.
//!
//! The Java `toMap`/`fromMap` were empty stubs — signature data was silently
//! dropped on serialization. This port implements them properly.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A detached signature over some value, describing who signed it and how.
///
/// This crate does not perform signing/verification; `Signature` is a metadata
/// record carried inside [`super::PublicKey`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Signature {
    /// The value that was signed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_signed: Option<String>,
    /// The signing algorithm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// When it was signed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_date: Option<DateTime<Utc>>,
    /// Username of the signer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by_username: Option<String>,
    /// Fingerprint of the signer's key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by_fingerprint: Option<String>,
    /// Address of the signer's key. Identity for equality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by_address: Option<String>,
}

impl PartialEq for Signature {
    fn eq(&self, other: &Self) -> bool {
        match (&self.signed_by_address, &other.signed_by_address) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Signature {}

impl std::hash::Hash for Signature {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.signed_by_address.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_on_address() {
        let mut a = Signature::default();
        let mut b = Signature::default();
        assert_ne!(a, b); // both None -> not equal, matching Java
        a.signed_by_address = Some("addr1".into());
        b.signed_by_address = Some("addr1".into());
        assert_eq!(a, b);
    }

    #[test]
    fn json_round_trip() {
        let s = Signature {
            value_signed: Some("hello".into()),
            algorithm: Some("Ed25519".into()),
            signed_date: Some(Utc::now()),
            signed_by_address: Some("addr".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(s.value_signed, back.value_signed);
        assert_eq!(s.signed_by_address, back.signed_by_address);
    }
}
