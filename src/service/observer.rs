//! Ports `ra.common.service.ServiceStatusObserver`.

use crate::service::ServiceStatus;

/// Notified whenever a service's status changes.
pub trait ServiceStatusObserver: Send + Sync {
    /// `service_full_name` changed to `status`.
    fn service_status_changed(&self, service_full_name: &str, status: ServiceStatus);
}
