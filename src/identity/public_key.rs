//! Ports `ra.common.identity.PublicKey`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::crypto::Addressable;
use crate::identity::Signature;

/// A public key plus how it is encoded and any (optionally signed) attributes.
///
/// The key material itself lives in [`PublicKey::address`] (the encoded key
/// string); the `is_*` flags say which encoding was used.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PublicKey {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Key type label (e.g. curve name).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub key_type: Option<String>,
    #[serde(default)]
    pub is_identity_key: bool,
    #[serde(default)]
    pub is_encryption_key: bool,
    #[serde(default)]
    pub is_base64_encoded: bool,
    #[serde(default)]
    pub is_base58_encoded: bool,
    #[serde(default)]
    pub is_pem: bool,
    #[serde(default)]
    pub is_hex: bool,
    /// Free-form attributes.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, Value>,
    /// Attribute name -> signatures over it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub signed_attributes: BTreeMap<String, Vec<Signature>>,
}

impl PublicKey {
    /// A key holding only its encoded address.
    pub fn from_address(address: impl Into<String>) -> Self {
        PublicKey {
            address: Some(address.into()),
            ..Default::default()
        }
    }

    /// Set an attribute.
    pub fn add_attribute(&mut self, name: impl Into<String>, value: Value) {
        self.attributes.insert(name.into(), value);
    }

    /// Get an attribute.
    pub fn attribute(&self, name: &str) -> Option<&Value> {
        self.attributes.get(name)
    }

    /// Append a signature to an attribute.
    pub fn add_signed_attribute(&mut self, name: impl Into<String>, signature: Signature) {
        self.signed_attributes.entry(name.into()).or_default().push(signature);
    }

    /// Remove any signature on `name` made by `signed_by_address`.
    pub fn remove_signature(&mut self, name: &str, signed_by_address: &str) {
        if let Some(sigs) = self.signed_attributes.get_mut(name) {
            sigs.retain(|s| s.signed_by_address.as_deref() != Some(signed_by_address));
        }
    }
}

impl Addressable for PublicKey {
    fn fingerprint(&self) -> Option<&str> {
        self.fingerprint.as_deref()
    }
    fn set_fingerprint(&mut self, fingerprint: Option<String>) {
        self.fingerprint = fingerprint;
    }
    fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }
    fn set_address(&mut self, address: Option<String>) {
        self.address = address;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_attributes() {
        let mut pk = PublicKey::from_address("B32ADDR");
        let sig = Signature {
            signed_by_address: Some("signer".into()),
            ..Default::default()
        };
        pk.add_signed_attribute("email", sig);
        assert_eq!(pk.signed_attributes["email"].len(), 1);
        pk.remove_signature("email", "signer");
        assert!(pk.signed_attributes["email"].is_empty());
    }

    #[test]
    fn json_round_trip() {
        let mut pk = PublicKey::from_address("addr");
        pk.is_identity_key = true;
        pk.add_attribute("k", Value::String("v".into()));
        let json = serde_json::to_string(&pk).unwrap();
        let back: PublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(back.address.as_deref(), Some("addr"));
        assert!(back.is_identity_key);
        assert_eq!(back.attribute("k").unwrap(), &Value::String("v".into()));
    }
}
