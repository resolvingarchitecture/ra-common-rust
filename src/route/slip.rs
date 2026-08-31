//! Ports `ra.common.route.{RoutingSlip, DynamicRoutingSlip}`.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::model::{Route, RouteMeta};

/// A LIFO stack of routes walked one hop at a time.
///
/// The Java version serialized its stack and rebuilt each entry reflectively
/// (iterating the list backwards to restore stack order). Here it is a plain
/// `VecDeque<Route>` round-trip; `push`/`next`/`peek` operate on the front.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicRoutingSlip {
    pub meta: RouteMeta,
    #[serde(default)]
    routes: VecDeque<Route>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    current_route: Option<Box<Route>>,
}

impl DynamicRoutingSlip {
    /// An empty slip.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a route onto the stack, stamping it with this slip's `route_id`
    /// (matching the Java `addRoute`).
    pub fn add_route(&mut self, mut route: Route) {
        route.set_route_id(self.meta.route_id);
        self.routes.push_front(route);
    }

    /// Number of routes not yet consumed.
    pub fn number_remaining_routes(&self) -> usize {
        self.routes.len()
    }

    /// The current route, advancing to the first one if not started.
    pub fn current_route(&mut self) -> Option<&Route> {
        if self.current_route.is_none() {
            self.next_route();
        }
        self.current_route.as_deref()
    }

    /// Pop and return the next route (now the current one), or `None` when the
    /// slip is exhausted.
    pub fn next_route(&mut self) -> Option<&Route> {
        self.current_route = self.routes.pop_front().map(Box::new);
        self.current_route.as_deref()
    }

    /// Look at the next route without consuming it.
    pub fn peek_at_next_route(&self) -> Option<&Route> {
        self.routes.front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::model::SimpleRoute;

    #[test]
    fn lifo_walk() {
        let mut slip = DynamicRoutingSlip::new();
        slip.add_route(Route::Simple(SimpleRoute::new("a", "op")));
        slip.add_route(Route::Simple(SimpleRoute::new("b", "op")));
        slip.add_route(Route::Simple(SimpleRoute::new("c", "op")));
        assert_eq!(slip.number_remaining_routes(), 3);

        // last pushed comes out first
        assert_eq!(slip.next_route().unwrap().service(), Some("c"));
        assert_eq!(slip.next_route().unwrap().service(), Some("b"));
        assert_eq!(slip.peek_at_next_route().unwrap().service(), Some("a"));
        assert_eq!(slip.next_route().unwrap().service(), Some("a"));
        assert!(slip.next_route().is_none());
    }

    #[test]
    fn add_route_stamps_route_id() {
        let mut slip = DynamicRoutingSlip::new();
        slip.add_route(Route::Simple(SimpleRoute::new("a", "op")));
        assert_eq!(slip.peek_at_next_route().unwrap().route_id(), slip.meta.route_id);
    }

    #[test]
    fn json_round_trip() {
        let mut slip = DynamicRoutingSlip::new();
        slip.add_route(Route::Simple(SimpleRoute::new("a", "op")));
        slip.add_route(Route::external("b", "op"));
        let json = serde_json::to_string(&slip).unwrap();
        let back: DynamicRoutingSlip = serde_json::from_str(&json).unwrap();
        assert_eq!(back.number_remaining_routes(), 2);
    }
}
