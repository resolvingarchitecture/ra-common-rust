//! Ports `ra.common.route.{ExternalRoute, SimpleExternalRoute, RelayedExternalRoute}`.

use serde::{Deserialize, Serialize};

use super::model::RouteMeta;
use crate::network::NetworkPeer;

/// Status codes an [`SimpleExternalRoute`] can carry (ports the `ExternalRoute`
/// `int` constants).
pub mod status {
    pub const DESTINATION_PEER_REQUIRED: i32 = 2;
    pub const DESTINATION_PEER_WRONG_NETWORK: i32 = 3;
    pub const DESTINATION_PEER_NOT_FOUND: i32 = 4;
    pub const NO_SERVICE: i32 = 7;
    pub const NO_OPERATION: i32 = 8;
    pub const NO_ADDRESS: i32 = 9;
    pub const NO_FINGERPRINT: i32 = 10;
    pub const NO_PORT: i32 = 11;
}

/// A route to a service on a remote peer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimpleExternalRoute {
    pub meta: RouteMeta,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origination: Option<NetworkPeer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<NetworkPeer>,
    #[serde(default)]
    pub send_content_only: bool,
    #[serde(default)]
    pub status_code: i32,
}

impl SimpleExternalRoute {
    /// A content-only external route to `operation` on `service`.
    pub fn new(service: impl Into<String>, operation: impl Into<String>) -> Self {
        SimpleExternalRoute {
            meta: RouteMeta::of(service, operation),
            send_content_only: true,
            ..Default::default()
        }
    }

    /// An external route with explicit origination / destination peers.
    pub fn with_peers(
        service: impl Into<String>,
        operation: impl Into<String>,
        origination: NetworkPeer,
        destination: NetworkPeer,
    ) -> Self {
        SimpleExternalRoute {
            meta: RouteMeta::of(service, operation),
            origination: Some(origination),
            destination: Some(destination),
            send_content_only: false,
            status_code: 0,
        }
    }
}

/// An external route relayed through intermediate peers, with delay/copy/
/// sensitivity controls.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelayedExternalRoute {
    pub base: SimpleExternalRoute,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_peer: Option<NetworkPeer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_peer: Option<NetworkPeer>,
    #[serde(default)]
    pub delayed: bool,
    #[serde(default)]
    pub min_delay: i64,
    #[serde(default)]
    pub max_delay: i64,
    #[serde(default)]
    pub copy: bool,
    #[serde(default)]
    pub min_copies: i32,
    #[serde(default)]
    pub max_copies: i32,
    #[serde(default)]
    pub sensitivity: i32,
}
