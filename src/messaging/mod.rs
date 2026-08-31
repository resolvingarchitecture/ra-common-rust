//! Messages carried inside an [`crate::envelope::Envelope`].
//!
//! Ports the `ra.common.messaging` package. The Java `Message` interface +
//! `BaseMessage` + concrete subclasses become a single `#[serde(tag = "kind")]`
//! enum.

pub mod channel;
pub mod command;
pub mod document;
pub mod email;
pub mod event;
pub mod text;

pub use channel::{MessageBus, MessageChannel, MessageConsumer, MessageProducer};
pub use command::{Command, CommandMessage};
pub use document::{DocumentMessage, CONTENT, ENTITY, EXCEPTIONS};
pub use email::Email;
pub use event::{EventMessage, EventType};
pub use text::TextMessage;

use serde::{Deserialize, Serialize};

/// Any message that can travel inside an [`crate::envelope::Envelope`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Message {
    Document(DocumentMessage),
    Command(CommandMessage),
    Event(EventMessage),
    /// Boxed: `TextMessage` holds two [`crate::identity::Did`]s and is much
    /// larger than the other variants.
    Text(Box<TextMessage>),
}

impl Message {
    /// Accumulated non-fatal error messages.
    pub fn error_messages(&self) -> &Vec<String> {
        match self {
            Message::Document(m) => &m.error_messages,
            Message::Command(m) => &m.error_messages,
            Message::Event(m) => &m.error_messages,
            Message::Text(m) => &m.error_messages,
        }
    }

    /// Append an error message.
    pub fn add_error_message(&mut self, msg: impl Into<String>) {
        let list = match self {
            Message::Document(m) => &mut m.error_messages,
            Message::Command(m) => &mut m.error_messages,
            Message::Event(m) => &mut m.error_messages,
            Message::Text(m) => &mut m.error_messages,
        };
        list.push(msg.into());
    }

    /// Clear all error messages.
    pub fn clear_error_messages(&mut self) {
        match self {
            Message::Document(m) => m.error_messages.clear(),
            Message::Command(m) => m.error_messages.clear(),
            Message::Event(m) => m.error_messages.clear(),
            Message::Text(m) => m.error_messages.clear(),
        }
    }

    /// Borrow the inner [`DocumentMessage`], if this is one.
    pub fn as_document(&self) -> Option<&DocumentMessage> {
        match self {
            Message::Document(m) => Some(m),
            _ => None,
        }
    }

    /// Mutably borrow the inner [`DocumentMessage`], if this is one.
    pub fn as_document_mut(&mut self) -> Option<&mut DocumentMessage> {
        match self {
            Message::Document(m) => Some(m),
            _ => None,
        }
    }

    /// Borrow the inner [`EventMessage`], if this is one.
    pub fn as_event(&self) -> Option<&EventMessage> {
        match self {
            Message::Event(m) => Some(m),
            _ => None,
        }
    }

    /// Borrow the inner [`CommandMessage`], if this is one.
    pub fn as_command(&self) -> Option<&CommandMessage> {
        match self {
            Message::Command(m) => Some(m),
            _ => None,
        }
    }
}

impl From<DocumentMessage> for Message {
    fn from(m: DocumentMessage) -> Self {
        Message::Document(m)
    }
}
impl From<CommandMessage> for Message {
    fn from(m: CommandMessage) -> Self {
        Message::Command(m)
    }
}
impl From<EventMessage> for Message {
    fn from(m: EventMessage) -> Self {
        Message::Event(m)
    }
}
impl From<TextMessage> for Message {
    fn from(m: TextMessage) -> Self {
        Message::Text(Box::new(m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tagged_round_trip() {
        for m in [
            Message::Document(DocumentMessage::new()),
            Message::Command(CommandMessage::new(Command::Start)),
            Message::Event(EventMessage::of(EventType::ServiceStatus)),
            Message::Text(Box::<TextMessage>::default()),
        ] {
            let json = serde_json::to_string(&m).unwrap();
            assert!(json.contains("\"kind\":"));
            let back: Message = serde_json::from_str(&json).unwrap();
            assert_eq!(
                std::mem::discriminant(&m),
                std::mem::discriminant(&back)
            );
        }
    }

    #[test]
    fn error_messages_shared_api() {
        let mut m = Message::Command(CommandMessage::new(Command::Report));
        m.add_error_message("boom");
        assert_eq!(m.error_messages(), &vec!["boom".to_string()]);
        m.clear_error_messages();
        assert!(m.error_messages().is_empty());
    }
}
