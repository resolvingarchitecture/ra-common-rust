//! The [`Route`] enum and its shared metadata.
//!
//! Ports `ra.common.route.{Route, BaseRoute, SimpleRoute}`.

use serde::{Deserialize, Serialize};

use super::external::{RelayedExternalRoute, SimpleExternalRoute};
use super::slip::DynamicRoutingSlip;
use crate::util::random::next_long;

/// Fields every route carries (`ra.common.route.BaseRoute`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default)]
    pub routed: bool,
    /// Correlates the routes of a single slip. Random at construction.
    pub route_id: i64,
}

impl Default for RouteMeta {
    fn default() -> Self {
        RouteMeta {
            service: None,
            operation: None,
            routed: false,
            route_id: next_long(),
        }
    }
}

impl RouteMeta {
    /// Meta for a `service` / `operation` pair.
    pub fn of(service: impl Into<String>, operation: impl Into<String>) -> Self {
        RouteMeta {
            service: Some(service.into()),
            operation: Some(operation.into()),
            ..Default::default()
        }
    }
}

/// A single in-process route: a `service` and an `operation` on it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SimpleRoute {
    pub meta: RouteMeta,
}

impl SimpleRoute {
    /// A route to `operation` on `service`.
    pub fn new(service: impl Into<String>, operation: impl Into<String>) -> Self {
        SimpleRoute {
            meta: RouteMeta::of(service, operation),
        }
    }
}

/// Any route variant. Ports the reflectively-reconstructed `Route` hierarchy as
/// a `#[serde(tag = "type")]` enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Route {
    Simple(SimpleRoute),
    RoutingSlip(DynamicRoutingSlip),
    SimpleExternal(Box<SimpleExternalRoute>),
    RelayedExternal(Box<RelayedExternalRoute>),
}

impl Route {
    /// A [`Route::Simple`] to `operation` on `service`.
    pub fn simple(service: impl Into<String>, operation: impl Into<String>) -> Route {
        Route::Simple(SimpleRoute::new(service, operation))
    }

    /// A [`Route::SimpleExternal`] to `operation` on `service`.
    pub fn external(service: impl Into<String>, operation: impl Into<String>) -> Route {
        Route::SimpleExternal(Box::new(SimpleExternalRoute::new(service, operation)))
    }

    /// Shared metadata.
    pub fn meta(&self) -> &RouteMeta {
        match self {
            Route::Simple(r) => &r.meta,
            Route::RoutingSlip(r) => &r.meta,
            Route::SimpleExternal(r) => &r.meta,
            Route::RelayedExternal(r) => &r.base.meta,
        }
    }

    /// Shared metadata, mutably.
    pub fn meta_mut(&mut self) -> &mut RouteMeta {
        match self {
            Route::Simple(r) => &mut r.meta,
            Route::RoutingSlip(r) => &mut r.meta,
            Route::SimpleExternal(r) => &mut r.meta,
            Route::RelayedExternal(r) => &mut r.base.meta,
        }
    }

    /// Target service name.
    pub fn service(&self) -> Option<&str> {
        self.meta().service.as_deref()
    }

    /// Target operation name.
    pub fn operation(&self) -> Option<&str> {
        self.meta().operation.as_deref()
    }

    /// Whether this route has been consumed.
    pub fn routed(&self) -> bool {
        self.meta().routed
    }

    /// Set the routed flag.
    pub fn set_routed(&mut self, routed: bool) {
        self.meta_mut().routed = routed;
    }

    /// The slip-correlation id.
    pub fn route_id(&self) -> i64 {
        self.meta().route_id
    }

    /// Set the slip-correlation id.
    pub fn set_route_id(&mut self, id: i64) {
        self.meta_mut().route_id = id;
    }
}

impl From<SimpleRoute> for Route {
    fn from(r: SimpleRoute) -> Self {
        Route::Simple(r)
    }
}

impl From<SimpleExternalRoute> for Route {
    fn from(r: SimpleExternalRoute) -> Self {
        Route::SimpleExternal(Box::new(r))
    }
}

impl From<RelayedExternalRoute> for Route {
    fn from(r: RelayedExternalRoute) -> Self {
        Route::RelayedExternal(Box::new(r))
    }
}
