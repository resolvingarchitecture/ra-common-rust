//! Routing: routes, routing slips and external/relayed routes.
//!
//! Ports the `ra.common.route` package.

pub mod external;
pub mod model;
pub mod slip;

pub use external::{status, RelayedExternalRoute, SimpleExternalRoute};
pub use model::{Route, RouteMeta, SimpleRoute};
pub use slip::DynamicRoutingSlip;
