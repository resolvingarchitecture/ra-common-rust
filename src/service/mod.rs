//! The service framework: the `Service` / `LifeCycle` contract, shared state,
//! status enums and reports.
//!
//! Ports the `ra.common.service` package (except `ServiceDaemon`, deferred).

pub mod base;
pub mod message;
pub mod observer;
pub mod report;
pub mod status;

pub use base::{Service, ServiceCore, RA_SERVICE_IMPL};
pub use message::{ServiceMessage, NO_ERROR, REQUEST_REQUIRED};
pub use observer::ServiceStatusObserver;
pub use report::ServiceReport;
pub use status::{ServiceLevel, ServiceStatus};
