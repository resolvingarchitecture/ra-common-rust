//! The `Service` contract and its shared state.
//!
//! Ports `ra.common.service.{Service, BaseService}`. Rust has no abstract
//! classes, so `BaseService`'s state becomes [`ServiceCore`] (which a concrete
//! service embeds) and its behaviour becomes default methods on the [`Service`]
//! trait.

use std::sync::Arc;

use crate::envelope::Envelope;
use crate::lifecycle::LifeCycle;
use crate::messaging::{Command, EventMessage, EventType, Message, MessageProducer};
use crate::service::{ServiceReport, ServiceStatus, ServiceStatusObserver};
use crate::Properties;

/// The service constant `ra.service.impl`, the config key naming the service
/// implementation class (used by the Java `ServiceDaemon`).
pub const RA_SERVICE_IMPL: &str = "ra.service.impl";

/// State shared by every service (ports the `BaseService` fields).
#[derive(Clone)]
pub struct ServiceCore {
    /// Fully-qualified name of the concrete service.
    pub service_class_name: String,
    /// Current status.
    pub status: ServiceStatus,
    /// Whether the service is registered with a registrar.
    pub registered: bool,
    /// Service version string.
    pub version: Option<String>,
    /// Names of services this one depends on.
    pub services_dependent_upon: Vec<String>,
    /// Effective configuration.
    pub config: Properties,
    /// Where this service sends outbound envelopes.
    pub producer: Option<Arc<dyn MessageProducer>>,
    /// Notified on status changes.
    pub observer: Option<Arc<dyn ServiceStatusObserver>>,
}

impl std::fmt::Debug for ServiceCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceCore")
            .field("service_class_name", &self.service_class_name)
            .field("status", &self.status)
            .field("registered", &self.registered)
            .field("version", &self.version)
            .field("services_dependent_upon", &self.services_dependent_upon)
            .field("has_producer", &self.producer.is_some())
            .field("has_observer", &self.observer.is_some())
            .finish()
    }
}

impl ServiceCore {
    /// New core for the given fully-qualified service name.
    pub fn new(service_class_name: impl Into<String>) -> Self {
        ServiceCore {
            service_class_name: service_class_name.into(),
            status: ServiceStatus::NotInitialized,
            registered: false,
            version: None,
            services_dependent_upon: Vec::new(),
            config: Properties::new(),
            producer: None,
            observer: None,
        }
    }

    /// Declare a dependency on another service.
    pub fn add_dependent_service(&mut self, name: impl Into<String>) {
        self.services_dependent_upon.push(name.into());
    }

    /// Send an envelope through the configured producer (no-op returning `false`
    /// if none is set).
    pub fn send(&self, envelope: Envelope) -> bool {
        match &self.producer {
            Some(p) => p.send(envelope),
            None => false,
        }
    }

    /// A fresh [`ServiceReport`] for the current state.
    pub fn report(&self) -> ServiceReport {
        ServiceReport {
            service_class_name: self.service_class_name.clone(),
            service_status: self.status,
            registered: self.registered,
            running: self.status == ServiceStatus::Running,
            version: self.version.clone(),
            services_dependent_upon: self.services_dependent_upon.clone(),
        }
    }

    /// Change status; if it actually changed, notify the observer and publish a
    /// `SERVICE_STATUS` event (routed to `ra.notification.NotificationService`).
    pub fn update_status(&mut self, status: ServiceStatus) {
        if self.status == status {
            return;
        }
        self.status = status;
        if let Some(obs) = &self.observer {
            obs.service_status_changed(&self.service_class_name, status);
        }
        if self.producer.is_some() {
            let mut ev = EventMessage::of(EventType::ServiceStatus);
            let _ = ev.set_message(&self.report());
            let mut e = Envelope::event(EventType::ServiceStatus);
            e.message = Some(Message::Event(ev));
            e.add_route("ra.notification.NotificationService", "PUBLISH");
            e.ratchet();
            self.send(e);
        }
    }
}

/// A message-driven service. Ports `ra.common.service.Service` + the reusable
/// parts of `BaseService`.
pub trait Service: LifeCycle {
    /// Borrow the shared state.
    fn core(&self) -> &ServiceCore;
    /// Borrow the shared state mutably.
    fn core_mut(&mut self) -> &mut ServiceCore;

    /// Handle a document message. Default: ignore.
    fn handle_document(&mut self, _envelope: &mut Envelope) {}

    /// Handle an event message. Default: ignore.
    fn handle_event(&mut self, _envelope: &mut Envelope) {}

    /// Handle a command message. Default: dispatch [`Command`] to the matching
    /// [`LifeCycle`] method.
    fn handle_command(&mut self, envelope: &mut Envelope) {
        let command = envelope
            .message
            .as_ref()
            .and_then(Message::as_command)
            .and_then(|c| c.command);
        let Some(command) = command else {
            return;
        };
        let config = self.core().config.clone();
        match command {
            Command::Start => {
                self.start(&config);
            }
            Command::Pause => {
                self.pause();
            }
            Command::Unpause => {
                self.unpause();
            }
            Command::Restart => {
                self.restart();
            }
            Command::Shutdown => {
                self.shutdown();
            }
            Command::GracefullyShutdown => {
                self.graceful_shutdown();
            }
            Command::Report => {
                let report = self.report();
                if let Ok(v) = serde_json::to_value(&report) {
                    // headers always accept a value regardless of message kind
                    envelope.set_header("result", v);
                }
            }
            Command::NetState
            | Command::RegisterStateChangeListener
            | Command::UnregisterStateChangeListener => {}
        }
    }

    /// Handle a headers-only envelope. Default: ignore.
    fn handle_headers(&mut self, _envelope: &mut Envelope) {}

    /// Current status.
    fn service_status(&self) -> ServiceStatus {
        self.core().status
    }

    /// A fresh report.
    fn report(&self) -> ServiceReport {
        self.core().report()
    }

    /// Dispatch `envelope` to the right handler by message kind and return it
    /// (for the caller to reply with). Ports `BaseService.receive`.
    fn handle(&mut self, mut envelope: Envelope) -> Envelope {
        match envelope.message.as_ref() {
            Some(Message::Document(_)) => self.handle_document(&mut envelope),
            Some(Message::Event(_)) => self.handle_event(&mut envelope),
            Some(Message::Command(_)) => self.handle_command(&mut envelope),
            _ => self.handle_headers(&mut envelope),
        }
        envelope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::CommandMessage;

    struct Toy {
        core: ServiceCore,
        started: bool,
    }

    impl LifeCycle for Toy {
        fn start(&mut self, _p: &Properties) -> bool {
            self.started = true;
            self.core.update_status(ServiceStatus::Running);
            true
        }
        fn shutdown(&mut self) -> bool {
            self.started = false;
            self.core.update_status(ServiceStatus::Shutdown);
            true
        }
    }

    impl Service for Toy {
        fn core(&self) -> &ServiceCore {
            &self.core
        }
        fn core_mut(&mut self) -> &mut ServiceCore {
            &mut self.core
        }
    }

    #[test]
    fn command_message_drives_lifecycle() {
        let mut toy = Toy {
            core: ServiceCore::new("ra.test.Toy"),
            started: false,
        };
        let mut e = Envelope::command();
        if let Some(Message::Command(c)) = e.message.as_mut() {
            *c = CommandMessage::new(Command::Start);
        }
        let out = toy.handle(e);
        assert!(toy.started);
        assert_eq!(toy.service_status(), ServiceStatus::Running);
        // Report command populates result
        let mut r = Envelope::command();
        if let Some(Message::Command(c)) = r.message.as_mut() {
            *c = CommandMessage::new(Command::Report);
        }
        let r = toy.handle(r);
        assert!(r.header("result").is_some());
        drop(out);
    }

    #[test]
    fn report_reflects_state() {
        let mut core = ServiceCore::new("ra.test.Svc");
        core.version = Some("9.9".into());
        core.update_status(ServiceStatus::Running);
        let rep = core.report();
        assert!(rep.running);
        assert_eq!(rep.version.as_deref(), Some("9.9"));
    }
}
