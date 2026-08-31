//! Crate-wide error type. Replaces the family of checked `*Exception` classes in
//! `ra-common-java` (`ServiceNotFoundException`, `FileCreationFailedException`, ...).

use std::fmt;

/// The result type used throughout this crate.
pub type Result<T> = std::result::Result<T, RaError>;

/// Everything that can go wrong in `ra_common`.
#[derive(Debug, thiserror::Error)]
pub enum RaError {
    /// A value could not be (de)serialized to/from JSON.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An I/O operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A string could not be decoded from its expected encoding (base32/58/hex/base64).
    #[error("decode error: {0}")]
    Decode(String),

    /// A cryptographic operation failed or verification did not pass.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// A malformed value was supplied (bad HashCash token, bad multihash, ...).
    #[error("invalid input: {0}")]
    Invalid(String),

    /// A named service could not be found.
    #[error("service not found: {0}")]
    ServiceNotFound(String),

    /// A service is registered but not reachable.
    #[error("service not accessible: {0}")]
    ServiceNotAccessible(String),

    /// A service type is not supported by this runtime.
    #[error("service not supported: {0}")]
    ServiceNotSupported(String),

    /// A service with this identity is already registered.
    #[error("service already registered: {0}")]
    ServiceAlreadyRegistered(String),

    /// A file could not be created.
    #[error("file creation failed: {0}")]
    FileCreationFailed(String),

    /// A file exists but is not readable.
    #[error("file not readable: {0}")]
    FileNotReadable(String),

    /// A file exists but is not writeable.
    #[error("file not writeable: {0}")]
    FileNotWriteable(String),
}

impl RaError {
    /// Build a [`RaError::Decode`] from anything displayable.
    pub fn decode(msg: impl fmt::Display) -> Self {
        RaError::Decode(msg.to_string())
    }

    /// Build a [`RaError::Crypto`] from anything displayable.
    pub fn crypto(msg: impl fmt::Display) -> Self {
        RaError::Crypto(msg.to_string())
    }

    /// Build a [`RaError::Invalid`] from anything displayable.
    pub fn invalid(msg: impl fmt::Display) -> Self {
        RaError::Invalid(msg.to_string())
    }
}
