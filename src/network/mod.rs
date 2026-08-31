//! Minimal network identity types needed by [`crate::route`] and
//! [`crate::envelope`].
//!
//! Ports `ra.common.network.Network`, `NetworkStatus` and `NetworkPeer`. The
//! full network service layer (`NetworkService`, sessions, `NetworkState`,
//! reports) is deferred to a later phase.

use serde::{Deserialize, Serialize};

use crate::identity::Did;

/// Transports a peer can be reached over. Ports `ra.common.network.Network`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Network {
    Card,
    Nfc,
    Http,
    Tor,
    I2p,
    WiFi,
    Bluetooth,
    Satellite,
    FsRadio,
    LiFi,
}

/// Connection state of a network. Ports `ra.common.network.NetworkStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkStatus {
    NotInstalled,
    Closed,
    Error,
    PortConflict,
    Waiting,
    Warmup,
    Connecting,
    Connected,
    Verified,
    Hanging,
    Failed,
    Blocked,
    Disconnected,
}

/// A peer in a peer-to-peer network, identified by [`Did`] and [`Network`].
///
/// Equality follows the Java version: two peers are equal iff both have a
/// public-key address and fingerprint and both match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPeer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub network: Network,
    pub did: Did,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
}

impl NetworkPeer {
    /// A new peer on `network` with a default [`Did`].
    pub fn new(network: Network) -> Self {
        NetworkPeer {
            id: None,
            network,
            did: Did::default(),
            port: None,
            services: Vec::new(),
        }
    }

    /// A new peer on `network` whose DID carries `username` / `passphrase`.
    pub fn with_credentials(
        network: Network,
        username: impl Into<String>,
        passphrase: Option<String>,
    ) -> Self {
        let mut did = Did::with_username(username);
        did.passphrase = passphrase;
        NetworkPeer {
            id: None,
            network,
            did,
            port: None,
            services: Vec::new(),
        }
    }
}

impl PartialEq for NetworkPeer {
    fn eq(&self, other: &Self) -> bool {
        let key = |p: &NetworkPeer| match (&p.did.public_key.address, &p.did.public_key.fingerprint) {
            (Some(a), Some(f)) => Some((a.clone(), f.clone())),
            _ => None,
        };
        match (key(self), key(other)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_needs_address_and_fingerprint() {
        let mut a = NetworkPeer::new(Network::Tor);
        let mut b = NetworkPeer::new(Network::Tor);
        assert_ne!(a, b);
        for p in [&mut a, &mut b] {
            p.did.public_key.address = Some("addr".into());
            p.did.public_key.fingerprint = Some("fp".into());
        }
        assert_eq!(a, b);
    }

    #[test]
    fn json_round_trip() {
        let mut p = NetworkPeer::new(Network::I2p);
        p.id = Some("peer-1".into());
        p.port = Some(7654);
        p.services = vec!["ra.http".into()];
        let json = serde_json::to_string(&p).unwrap();
        let back: NetworkPeer = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id.as_deref(), Some("peer-1"));
        assert_eq!(back.port, Some(7654));
        assert_eq!(back.network, Network::I2p);
    }
}
