//! Ports `ra.common.messaging.DocumentMessage`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Keys used by [`crate::envelope::Envelope`] within `data[0]`.
pub const CONTENT: &str = "CONTENT";
/// Key for a domain entity payload.
pub const ENTITY: &str = "ENTITY";
/// Key for the list of accumulated exceptions.
pub const EXCEPTIONS: &str = "EXCEPTIONS";

/// A message carrying one or more named-value payload buckets. `data[0]` is the
/// primary bucket used by the `Envelope` content/entity/NVP helpers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMessage {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub error_messages: Vec<String>,
    pub data: Vec<Map<String, Value>>,
}

impl Default for DocumentMessage {
    fn default() -> Self {
        DocumentMessage {
            error_messages: Vec::new(),
            data: vec![Map::new()],
        }
    }
}

impl DocumentMessage {
    /// A document message with a single empty bucket.
    pub fn new() -> Self {
        Self::default()
    }

    /// The primary (`data[0]`) bucket, creating it if the vec was emptied.
    pub fn primary(&mut self) -> &mut Map<String, Value> {
        if self.data.is_empty() {
            self.data.push(Map::new());
        }
        &mut self.data[0]
    }

    /// Read a value from the primary bucket.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.first().and_then(|m| m.get(key))
    }

    /// Put a value into the primary bucket.
    pub fn put(&mut self, key: impl Into<String>, value: Value) {
        self.primary().insert(key.into(), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_bucket_exists() {
        let mut d = DocumentMessage::new();
        assert_eq!(d.data.len(), 1);
        d.put("k", Value::from(1));
        assert_eq!(d.get("k"), Some(&Value::from(1)));
    }
}
