//! Ports `ra.common.messaging.Email`.

use serde::{Deserialize, Serialize};

/// `text/plain` MIME type constant.
pub const MIMETYPE_TEXT_PLAIN: &str = "text/plain";

/// A simple email record. Not a [`super::Message`] — a persistable value type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub message_type: String,
    #[serde(default)]
    pub flag: i32,
}

impl Default for Email {
    fn default() -> Self {
        Email {
            id: None,
            to: None,
            from: None,
            subject: None,
            message: None,
            message_type: MIMETYPE_TEXT_PLAIN.to_string(),
            flag: 0,
        }
    }
}

impl Email {
    /// An anonymous message (no `from`).
    pub fn anonymous(to: impl Into<String>, subject: impl Into<String>, message: impl Into<String>) -> Self {
        Email {
            to: Some(to.into()),
            subject: Some(subject.into()),
            message: Some(message.into()),
            ..Default::default()
        }
    }

    /// A message with a sender.
    pub fn new(
        to: impl Into<String>,
        from: impl Into<String>,
        subject: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Email {
            to: Some(to.into()),
            from: Some(from.into()),
            subject: Some(subject.into()),
            message: Some(message.into()),
            ..Default::default()
        }
    }
}
