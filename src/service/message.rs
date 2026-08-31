//! Ports `ra.common.service.ServiceMessage`.

use serde::{Deserialize, Serialize};

/// Sentinel status code meaning "no error".
pub const NO_ERROR: i32 = -1;
/// Sentinel status code meaning "a request is required".
pub const REQUEST_REQUIRED: i32 = 0;

/// Base shape for service request/response messages: a status code plus optional
/// error text. Concrete services extend this with their own fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMessage {
    pub status_code: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// A stringified exception, if any (the Java field carried an `Exception`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exception: Option<String>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

impl Default for ServiceMessage {
    fn default() -> Self {
        ServiceMessage {
            status_code: NO_ERROR,
            error_message: None,
            exception: None,
            kind: None,
        }
    }
}

impl ServiceMessage {
    /// Whether this message represents an error.
    pub fn is_error(&self) -> bool {
        self.status_code != NO_ERROR || self.error_message.is_some()
    }
}
