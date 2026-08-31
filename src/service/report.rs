//! Ports `ra.common.service.ServiceReport`.

use serde::{Deserialize, Serialize};

use crate::service::ServiceStatus;

/// A snapshot of a service's health, emitted with `SERVICE_STATUS` events and
/// returned by `Service::report`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceReport {
    pub service_class_name: String,
    pub service_status: ServiceStatus,
    #[serde(default)]
    pub registered: bool,
    #[serde(default)]
    pub running: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services_dependent_upon: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let r = ServiceReport {
            service_class_name: "ra.http.HttpService".into(),
            service_status: ServiceStatus::Running,
            registered: true,
            running: true,
            version: Some("0.1.0".into()),
            services_dependent_upon: vec!["ra.tor.TorService".into()],
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: ServiceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.service_class_name, r.service_class_name);
        assert_eq!(back.service_status, ServiceStatus::Running);
        assert!(back.running);
    }
}
