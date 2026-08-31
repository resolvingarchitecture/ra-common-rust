//! Collection helpers.
//!
//! - `ra.common.LHMCache` (an access-ordered `LinkedHashMap` used as an LRU) maps
//!   directly onto [`lru::LruCache`], re-exported here.
//! - `ra.common.Stack` / `ra.common.DequeStack` (a LIFO stack used by routing
//!   slips) map onto [`std::collections::VecDeque`]; push with `push_front`, pop
//!   with `pop_front`, peek with `front`.

pub use lru::LruCache;

/// The routing-slip stack type. LIFO via `push_front` / `pop_front` / `front`.
pub type Stack<T> = std::collections::VecDeque<T>;
