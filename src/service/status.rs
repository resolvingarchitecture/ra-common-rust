//! Ports `ra.common.service.{ServiceStatus, ServiceLevel}`.

use serde::{Deserialize, Serialize};

/// Delivery guarantee for an envelope. Ports `ServiceLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ServiceLevel {
    /// May be lost, never duplicated or redelivered.
    AtMostOnce,
    /// Never lost, may be duplicated.
    #[default]
    AtLeastOnce,
    /// Processed exactly once.
    ExactlyOnce,
}

/// Detailed lifecycle state of a service. Ports the 20-value `ServiceStatus`
/// enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// Initial state.
    NotInitialized,
    /// Initializing configuration.
    Initializing,
    /// Waiting on a dependency to reach `Running`.
    Waiting,
    /// Starting up.
    Starting,
    /// Running normally.
    Running,
    /// Confirmed running by a received message.
    Verified,
    /// Running, though not everything is up (expected to be normal).
    PartiallyRunning,
    /// Running in a degraded manner; likely self-heals.
    DegradedRunning,
    /// Running but unstable; likely needs a restart.
    Unstable,
    /// Beginning to queue new requests.
    Pausing,
    /// Queueing new requests; pre-pause requests done.
    Paused,
    /// Resuming normal operation.
    Unpausing,
    /// Teardown imminent and not clean.
    ShuttingDown,
    /// Ideal clean teardown in progress.
    GracefullyShuttingDown,
    /// Was torn down forcefully; state may be corrupt.
    Shutdown,
    /// Shutdown was graceful; state is safe.
    GracefullyShutdown,
    /// Graceful shutdown then re-init.
    Restarting,
    /// No network available (not installed or off).
    Unavailable,
    /// Likely needs a restart.
    Error,
}

impl ServiceStatus {
    /// Whether the service is in a usable running state.
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            ServiceStatus::Running
                | ServiceStatus::Verified
                | ServiceStatus::PartiallyRunning
                | ServiceStatus::DegradedRunning
        )
    }
}
