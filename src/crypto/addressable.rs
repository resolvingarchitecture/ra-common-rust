//! Ports `ra.common.crypto.Addressable`.

/// Something reachable on a network by its public key: it exposes a `fingerprint`
/// and an `address`.
pub trait Addressable {
    /// Short fingerprint of the key.
    fn fingerprint(&self) -> Option<&str>;
    /// Set the fingerprint.
    fn set_fingerprint(&mut self, fingerprint: Option<String>);
    /// Network address (the encoded public key).
    fn address(&self) -> Option<&str>;
    /// Set the address.
    fn set_address(&mut self, address: Option<String>);
}
