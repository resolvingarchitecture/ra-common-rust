//! Lifecycle and status contracts shared by services and long-lived components.
//!
//! Ports `ra.common.LifeCycle`, `ra.common.Status` and `ra.common.Client`.

use crate::envelope::Envelope;
use crate::Properties;

/// Coarse run state of a component. Ports `ra.common.Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Status {
    Initialized,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Errored,
}

/// Start / pause / restart / shutdown contract. Ports `ra.common.LifeCycle`.
///
/// Every method returns `true` on success, matching the Java API. `unpause` is
/// named as such (rather than `resume`) to mirror the Java note about the
/// `Thread.resume` clash.
pub trait LifeCycle {
    /// Start the component with the given configuration.
    fn start(&mut self, properties: &Properties) -> bool;

    /// Begin queueing new work; let in-flight work finish.
    fn pause(&mut self) -> bool {
        false
    }

    /// Resume normal operation after [`LifeCycle::pause`].
    fn unpause(&mut self) -> bool {
        false
    }

    /// Graceful shutdown followed by start.
    fn restart(&mut self) -> bool {
        false
    }

    /// Teardown is imminent and may not be clean.
    fn shutdown(&mut self) -> bool;

    /// Ideal clean teardown.
    fn graceful_shutdown(&mut self) -> bool {
        self.shutdown()
    }
}

/// A caller that a service can send a reply [`Envelope`] back to.
///
/// Ports `ra.common.Client`.
pub trait Client: Send + Sync {
    /// Deliver a reply to the client.
    fn reply(&self, envelope: Envelope);
}
