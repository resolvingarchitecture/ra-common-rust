//! Producer / consumer / channel / bus contracts.
//!
//! Ports `ra.common.messaging.{MessageProducer, MessageConsumer, MessageChannel,
//! MessageBus}`. Concrete broker implementations live outside this crate.

use std::sync::Arc;

use crate::envelope::Envelope;
use crate::lifecycle::{Client, LifeCycle};
use crate::service::ServiceLevel;

/// Sends envelopes onward. Ports `MessageProducer`.
pub trait MessageProducer: Send + Sync {
    /// Send `envelope`. Returns `true` if accepted.
    fn send(&self, envelope: Envelope) -> bool;

    /// Send `envelope`, delivering the reply to `callback`.
    fn send_with_callback(&self, envelope: Envelope, callback: Arc<dyn Client>) -> bool;

    /// Route `envelope` to the dead-letter sink.
    fn dead_letter(&self, envelope: Envelope) -> bool;
}

/// Receives envelopes. Ports `MessageConsumer`.
pub trait MessageConsumer: Send + Sync {
    /// Handle `envelope`. Returns `true` if handled.
    fn receive(&mut self, envelope: Envelope) -> bool;
}

/// A named, bounded queue between stages. Ports `MessageChannel`.
pub trait MessageChannel: MessageProducer + LifeCycle {
    /// The channel name.
    fn name(&self) -> &str;
    /// Whether this channel fans out to subscribers.
    fn is_pub_sub(&self) -> bool;
    /// Number of envelopes currently queued.
    fn queued(&self) -> usize;
    /// Blocking receive.
    fn receive(&self) -> Option<Envelope>;
    /// Receive, waiting at most `timeout_ms` milliseconds.
    fn receive_timeout(&self, timeout_ms: u64) -> Option<Envelope>;
    /// Non-blocking receive.
    fn poll(&self) -> Option<Envelope>;
    /// Acknowledge processing of `envelope`.
    fn ack(&self, envelope: &Envelope);
}

/// Registers channels and publishes envelopes across them. Ports `MessageBus`.
pub trait MessageBus: LifeCycle {
    /// Register (or fetch) a channel by name.
    fn register_channel(&mut self, name: &str, service_level: ServiceLevel) -> bool;
    /// Publish `envelope` onto its routed channel.
    fn publish(&self, envelope: Envelope) -> bool;
    /// Publish `envelope` with a reply `callback`.
    fn publish_with_callback(&self, envelope: Envelope, callback: Arc<dyn Client>) -> bool;
    /// Mark `envelope` fully processed.
    fn completed(&self, envelope: &Envelope) -> bool;
}
