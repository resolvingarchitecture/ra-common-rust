//! The universal message wrapper passed between services.
//!
//! Ports `ra.common.Envelope` (and folds in the useful parts of the deprecated
//! `ra.common.DLC` static helpers as methods).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::file::Multipart;
use crate::identity::Did;
use crate::messaging::{
    CommandMessage, DocumentMessage, EventMessage, EventType, Message, TextMessage, CONTENT, ENTITY,
    EXCEPTIONS,
};
use crate::route::{DynamicRoutingSlip, Route};
use crate::service::ServiceLevel;

/// `Authorization` header name.
pub const HEADER_AUTHORIZATION: &str = "Authorization";
/// `Content-Disposition` header name.
pub const HEADER_CONTENT_DISPOSITION: &str = "Content-Disposition";
/// `Content-Transfer-Encoding` header name.
pub const HEADER_CONTENT_TRANSFER_ENCODING: &str = "Content-Transfer-Encoding";
/// `Content-Type` header name.
pub const HEADER_CONTENT_TYPE: &str = "Content-Type";
/// The JSON content type.
pub const HEADER_CONTENT_TYPE_JSON: &str = "application/json";
/// `User-Agent` header name.
pub const HEADER_USER_AGENT: &str = "User-Agent";

/// Which kind of [`Message`] an envelope carries. Ports `Envelope.MessageType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Document,
    Text,
    Event,
    Command,
    None,
}

/// The REST-ish verb an envelope represents. Ports `Envelope.Action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Post,
    Put,
    Delete,
    Get,
}

/// Wraps everything passed around the application so there is always a place for
/// header/routing metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub id: String,
    pub dynamic_routing_slip: DynamicRoutingSlip,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<Route>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<String>,
    pub did: Did,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    #[serde(default)]
    pub reply_to_client: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_reply_action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multipart: Option<Multipart>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_path: Option<String>,
    #[serde(default)]
    pub headers: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
    pub sensitivity: i32,
    #[serde(default)]
    pub delayed: bool,
    pub min_delay: i32,
    pub max_delay: i32,
    #[serde(default)]
    pub copy: bool,
    pub max_copies: i32,
    pub min_copies: i32,
    pub service_level: ServiceLevel,
}

impl Default for Envelope {
    fn default() -> Self {
        Envelope {
            id: Uuid::new_v4().to_string(),
            dynamic_routing_slip: DynamicRoutingSlip::new(),
            route: None,
            markers: Vec::new(),
            did: Did::default(),
            client: None,
            reply_to_client: false,
            client_reply_action: None,
            url: None,
            multipart: None,
            action: None,
            command_path: None,
            headers: Map::new(),
            message: None,
            sensitivity: 1,
            delayed: false,
            min_delay: 0,
            max_delay: 0,
            copy: false,
            max_copies: 0,
            min_copies: 0,
            service_level: ServiceLevel::AtLeastOnce,
        }
    }
}

impl Envelope {
    fn with_message(id: String, message: Option<Message>) -> Self {
        Envelope {
            id,
            message,
            ..Default::default()
        }
    }

    /// New envelope carrying a [`CommandMessage`].
    pub fn command() -> Self {
        Self::with_message(
            Uuid::new_v4().to_string(),
            Some(Message::Command(CommandMessage::default())),
        )
    }

    /// New envelope carrying an empty [`DocumentMessage`].
    pub fn document() -> Self {
        Self::with_message(
            Uuid::new_v4().to_string(),
            Some(Message::Document(DocumentMessage::new())),
        )
    }

    /// New document envelope with a caller-supplied id.
    pub fn document_with_id(id: impl Into<String>) -> Self {
        Self::with_message(id.into(), Some(Message::Document(DocumentMessage::new())))
    }

    /// New envelope with headers only and no message.
    pub fn headers_only() -> Self {
        Self::with_message(Uuid::new_v4().to_string(), None)
    }

    /// New envelope carrying an [`EventMessage`] of `event_type`.
    pub fn event(event_type: EventType) -> Self {
        Self::with_message(
            Uuid::new_v4().to_string(),
            Some(Message::Event(EventMessage::of(event_type))),
        )
    }

    /// New envelope carrying an empty [`TextMessage`].
    pub fn text() -> Self {
        Self::with_message(
            Uuid::new_v4().to_string(),
            Some(Message::Text(Box::<TextMessage>::default())),
        )
    }

    // ---- headers ---------------------------------------------------------

    /// Set a header value.
    pub fn set_header(&mut self, name: impl Into<String>, value: Value) {
        self.headers.insert(name.into(), value);
    }

    /// Whether a header is present.
    pub fn header_exists(&self, name: &str) -> bool {
        self.headers.contains_key(name)
    }

    /// Remove a header.
    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(name);
    }

    /// Get a header value.
    pub fn header(&self, name: &str) -> Option<&Value> {
        self.headers.get(name)
    }

    /// The `Content-Type` header, if set and a string.
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get(HEADER_CONTENT_TYPE).and_then(Value::as_str)
    }

    /// Set the `Content-Type` header.
    pub fn set_content_type(&mut self, content_type: impl Into<String>) {
        self.headers
            .insert(HEADER_CONTENT_TYPE.to_string(), Value::String(content_type.into()));
    }

    // ---- routing --------------------------------------------------------

    /// The route currently being processed. If none is set, falls back to the
    /// slip's current route.
    pub fn route(&mut self) -> Option<&Route> {
        if self.route.is_none() {
            if let Some(r) = self.dynamic_routing_slip.current_route() {
                self.route = Some(r.clone());
            }
        }
        self.route.as_ref()
    }

    /// Advance to the next route in the slip (`ratchet`).
    pub fn ratchet(&mut self) {
        self.route = self.dynamic_routing_slip.next_route().cloned();
    }

    /// Push an in-process route (`service` / `operation`) onto the slip.
    pub fn add_route(&mut self, service: impl Into<String>, operation: impl Into<String>) {
        self.dynamic_routing_slip
            .add_route(Route::simple(service, operation));
    }

    /// Push an external route (`service` / `operation`) onto the slip.
    pub fn add_external_route(&mut self, service: impl Into<String>, operation: impl Into<String>) {
        self.dynamic_routing_slip
            .add_route(Route::external(service, operation));
    }

    // ---- document payload accessors ------------------------------------

    fn doc_mut(&mut self) -> Option<&mut DocumentMessage> {
        self.message.as_mut().and_then(Message::as_document_mut)
    }

    fn doc(&self) -> Option<&DocumentMessage> {
        self.message.as_ref().and_then(Message::as_document)
    }

    /// Put a value under `CONTENT`. Returns `false` if not a document message.
    pub fn add_content(&mut self, content: Value) -> bool {
        match self.doc_mut() {
            Some(d) => {
                d.put(CONTENT, content);
                true
            }
            None => false,
        }
    }

    /// Read the `CONTENT` value.
    pub fn content(&self) -> Option<&Value> {
        self.doc().and_then(|d| d.get(CONTENT))
    }

    /// Put a value under `ENTITY`. Returns `false` if not a document message.
    pub fn add_entity(&mut self, entity: Value) -> bool {
        match self.doc_mut() {
            Some(d) => {
                d.put(ENTITY, entity);
                true
            }
            None => false,
        }
    }

    /// Read the `ENTITY` value.
    pub fn entity(&self) -> Option<&Value> {
        self.doc().and_then(|d| d.get(ENTITY))
    }

    /// Append a string to the `EXCEPTIONS` list. Returns `false` if not a
    /// document message.
    pub fn add_exception(&mut self, message: impl Into<String>) -> bool {
        let Some(d) = self.doc_mut() else {
            return false;
        };
        let entry = Value::String(message.into());
        match d.primary().get_mut(EXCEPTIONS) {
            Some(Value::Array(a)) => a.push(entry),
            _ => {
                d.primary()
                    .insert(EXCEPTIONS.to_string(), Value::Array(vec![entry]));
            }
        }
        true
    }

    /// The `EXCEPTIONS` list (empty if none / not a document message).
    pub fn exceptions(&self) -> Vec<String> {
        self.doc()
            .and_then(|d| d.get(EXCEPTIONS))
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    }

    /// Append a non-fatal error message to the underlying message.
    pub fn add_error_message(&mut self, message: impl Into<String>) {
        if let Some(m) = self.message.as_mut() {
            m.add_error_message(message);
        }
    }

    /// The underlying message's error messages.
    pub fn error_messages(&self) -> Vec<String> {
        self.message
            .as_ref()
            .map(|m| m.error_messages().clone())
            .unwrap_or_default()
    }

    /// Put a named value into the primary document bucket. Returns `false` if not
    /// a document message.
    pub fn add_nvp(&mut self, name: impl Into<String>, value: Value) -> bool {
        match self.doc_mut() {
            Some(d) => {
                d.put(name, value);
                true
            }
            None => false,
        }
    }

    /// Read a named value from the primary document bucket.
    pub fn value(&self, name: &str) -> Option<&Value> {
        self.doc().and_then(|d| d.get(name))
    }

    /// All named values in the primary document bucket.
    pub fn values(&self) -> Option<&Map<String, Value>> {
        self.doc().and_then(|d| d.data.first())
    }

    // ---- markers -------------------------------------------------------

    /// Whether `marker` is present.
    pub fn marker_present(&self, marker: &str) -> bool {
        self.markers.iter().any(|m| m == marker)
    }

    /// Add a marker.
    pub fn mark(&mut self, marker: impl Into<String>) {
        self.markers.push(marker.into());
    }

    // ---- serialization ------------------------------------------------

    /// Serialize to pretty JSON.
    ///
    /// # Errors
    /// [`crate::error::RaError::Json`] on failure.
    pub fn to_json(&self) -> crate::error::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON.
    ///
    /// # Errors
    /// [`crate::error::RaError::Json`] on failure.
    pub fn from_json(json: &str) -> crate::error::Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

impl PartialEq for Envelope {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Envelope {}

impl std::hash::Hash for Envelope {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factories_set_message_kind() {
        assert!(Envelope::document().message.unwrap().as_document().is_some());
        assert!(Envelope::command().message.unwrap().as_command().is_some());
        assert!(Envelope::event(EventType::BusStatus).message.unwrap().as_event().is_some());
        assert!(Envelope::headers_only().message.is_none());
    }

    #[test]
    fn content_round_trips_through_document() {
        let mut e = Envelope::document();
        assert!(e.add_content(Value::String("hello".into())));
        assert_eq!(e.content(), Some(&Value::String("hello".into())));

        let mut cmd = Envelope::command();
        assert!(!cmd.add_content(Value::Null));
    }

    #[test]
    fn exceptions_accumulate() {
        let mut e = Envelope::document();
        e.add_exception("first");
        e.add_exception("second");
        assert_eq!(e.exceptions(), vec!["first", "second"]);
    }

    #[test]
    fn ratchet_walks_the_slip() {
        let mut e = Envelope::document();
        e.add_route("ra.a.ServiceA", "OP");
        e.add_route("ra.b.ServiceB", "OP");
        e.ratchet();
        assert_eq!(e.route.as_ref().unwrap().service(), Some("ra.b.ServiceB"));
        e.ratchet();
        assert_eq!(e.route.as_ref().unwrap().service(), Some("ra.a.ServiceA"));
    }

    #[test]
    fn json_round_trip() {
        let mut e = Envelope::document();
        e.set_content_type(HEADER_CONTENT_TYPE_JSON);
        e.add_content(Value::from(42));
        e.add_route("ra.x.Svc", "DO");
        e.mark("seen");
        let json = e.to_json().unwrap();
        let back = Envelope::from_json(&json).unwrap();
        assert_eq!(back.id, e.id);
        assert_eq!(back.content_type(), Some(HEADER_CONTENT_TYPE_JSON));
        assert_eq!(back.content(), Some(&Value::from(42)));
        assert!(back.marker_present("seen"));
        assert_eq!(back.dynamic_routing_slip.number_remaining_routes(), 1);
    }

    #[test]
    fn equality_is_by_id() {
        let a = Envelope::document_with_id("same");
        let b = Envelope::document_with_id("same");
        assert_eq!(a, b);
    }
}
