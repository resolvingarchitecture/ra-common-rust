//! Ports `ra.common.messaging.TextMessage`.

use serde::{Deserialize, Serialize};

use crate::identity::Did;

/// A plain text message between two identities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextMessage {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub error_messages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<Did>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Did>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl TextMessage {
    /// A message from `from` to `to` carrying `text`.
    pub fn new(to: Did, from: Did, text: impl Into<String>) -> Self {
        TextMessage {
            error_messages: Vec::new(),
            to: Some(to),
            from: Some(from),
            text: Some(text.into()),
        }
    }
}
