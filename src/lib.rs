//! # ra-common
//!
//! A Rust port of [`ra-common-java`](https://github.com/resolvingarchitecture/ra-common-java),
//! the foundational types for the Resolving Architecture / 1M5 ecosystem.
//!
//! Serialization is [`serde`]-based and **not** wire-compatible with the Java
//! library (which used a hand-rolled JSON layer and reflective polymorphism).
//!
//! ```
//! use ra_common::envelope::Envelope;
//! use ra_common::serde_json::json;
//!
//! let mut e = Envelope::document();
//! e.add_route("ra.http.HttpService", "SEND");
//! e.add_content(json!({ "hello": "world" }));
//! e.ratchet();
//!
//! assert_eq!(e.route().unwrap().service(), Some("ra.http.HttpService"));
//! assert_eq!(e.content().unwrap()["hello"], "world");
//!
//! let json = e.to_json().unwrap();
//! let back = Envelope::from_json(&json).unwrap();
//! assert_eq!(back, e);
//! ```
//!
//! ## Phase 1 scope
//!
//! Implemented: [`envelope`], [`messaging`], [`route`], [`service`],
//! [`identity`], [`crypto`], [`content`], [`tasks`], [`config`], a minimal
//! [`network`] slice, and [`util`].
//!
//! Deferred: currency, locale/i18n, the full network service layer, `Protocol`,
//! shell/file/browser utilities, `InfoVault`.

#![forbid(unsafe_code)]

/// Re-export so downstream crates and doctests can name `serde_json::Value`
/// without a direct dependency.
pub use serde_json;

pub mod config;
pub mod content;
pub mod crypto;
pub mod encoding;
pub mod envelope;
pub mod error;
pub mod file;
pub mod identity;
pub mod lifecycle;
pub mod messaging;
pub mod network;
pub mod route;
pub mod service;
pub mod tasks;
pub mod util;

/// A bag of string configuration values. Ports `java.util.Properties`.
pub type Properties = std::collections::HashMap<String, String>;

pub use crate::error::{RaError, Result};
pub use crate::envelope::Envelope;
pub use crate::lifecycle::{Client, LifeCycle, Status};
pub use crate::messaging::Message;
pub use crate::route::Route;
pub use crate::service::{Service, ServiceCore, ServiceStatus};

/// Common imports for working with this crate.
pub mod prelude {
    pub use crate::content::{Content, ContentKind};
    pub use crate::crypto::{Hash, HashAlgorithm, HashCash, Multihash};
    pub use crate::envelope::{Action, Envelope, MessageType};
    pub use crate::error::{RaError, Result};
    pub use crate::identity::{Did, PublicKey, Signature};
    pub use crate::lifecycle::{Client, LifeCycle, Status};
    pub use crate::messaging::{
        Command, CommandMessage, DocumentMessage, EventMessage, EventType, Message, TextMessage,
    };
    pub use crate::network::{Network, NetworkPeer, NetworkStatus};
    pub use crate::route::{DynamicRoutingSlip, Route, SimpleRoute};
    pub use crate::service::{Service, ServiceCore, ServiceLevel, ServiceReport, ServiceStatus};
    pub use crate::tasks::{Task, TaskConfig, TaskRunner};
    pub use crate::Properties;
}
