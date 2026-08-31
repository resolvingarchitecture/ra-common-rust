//! Ports `ra.common.messaging.EventMessage`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Well-known event categories. Ports `EventMessage.Type`. The wire form is a
/// free-form string (`event_type`) so callers may use their own categories too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    Error,
    Exception,
    BusStatus,
    PeerStatus,
    ServiceStatus,
    DidStatus,
    NetworkStateUpdate,
    PriceChange,
}

impl EventType {
    /// The `SCREAMING_SNAKE` name used by the Java enum.
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::Error => "ERROR",
            EventType::Exception => "EXCEPTION",
            EventType::BusStatus => "BUS_STATUS",
            EventType::PeerStatus => "PEER_STATUS",
            EventType::ServiceStatus => "SERVICE_STATUS",
            EventType::DidStatus => "DID_STATUS",
            EventType::NetworkStateUpdate => "NETWORK_STATE_UPDATE",
            EventType::PriceChange => "PRICE_CHANGE",
        }
    }
}

/// An event notification with an optional structured payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub error_messages: Vec<String>,
    pub id: String,
    /// The event category (usually an [`EventType::as_str`] value).
    pub event_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Arbitrary structured payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Value>,
}

impl EventMessage {
    /// A new event of the given type with a fresh id.
    pub fn new(event_type: impl Into<String>) -> Self {
        EventMessage {
            error_messages: Vec::new(),
            id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            name: None,
            message: None,
        }
    }

    /// A new event using a well-known [`EventType`].
    pub fn of(event_type: EventType) -> Self {
        Self::new(event_type.as_str())
    }

    /// Attach a structured payload (anything `Serialize`).
    ///
    /// # Errors
    /// [`crate::error::RaError::Json`] if serialization fails.
    pub fn set_message<T: Serialize>(&mut self, message: &T) -> crate::error::Result<()> {
        self.message = Some(serde_json::to_value(message)?);
        Ok(())
    }
}
