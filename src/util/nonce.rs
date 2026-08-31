//! Replay protection via a bounded set of seen ids. Ports `ra.common.Nonce`.
//!
//! The Java version stored ids in an `ArrayList` and scanned it linearly; here
//! the membership check is O(1) (`HashSet`) with a parallel `VecDeque` tracking
//! insertion order for pruning. The Java pruning bug
//! (`max * (pct / 100)` integer-divides to zero) is fixed: we compute
//! `max * pct / 100`.

use std::collections::{HashSet, VecDeque};

/// Tracks recently-seen ids and rejects duplicates.
#[derive(Debug, Clone)]
pub struct Nonce {
    seen: HashSet<u64>,
    order: VecDeque<u64>,
    max_size: usize,
    prune_percent: u8,
}

impl Default for Nonce {
    /// 1,000,000 max entries, pruned 10% at a time (matching the Java defaults).
    fn default() -> Self {
        Nonce::new(1_000_000, 10)
    }
}

impl Nonce {
    /// A nonce set holding up to `max_size` ids, pruning `prune_percent` (clamped
    /// to `0..=100`) of `max_size` oldest entries once full.
    pub fn new(max_size: usize, prune_percent: u8) -> Self {
        Nonce {
            seen: HashSet::new(),
            order: VecDeque::new(),
            max_size,
            prune_percent: prune_percent.min(100),
        }
    }

    /// Register `id`. Returns `true` if it is new (processing should continue),
    /// `false` if it has been seen before (a replay).
    pub fn continue_on(&mut self, id: u64) -> bool {
        self.prune();
        if self.seen.contains(&id) {
            return false;
        }
        self.seen.insert(id);
        self.order.push_back(id);
        true
    }

    /// Number of ids currently tracked.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Whether no ids are tracked.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    fn prune(&mut self) {
        if self.order.len() <= self.max_size {
            return;
        }
        if self.prune_percent == 100 {
            self.seen.clear();
            self.order.clear();
            return;
        }
        let to_prune = self.max_size * self.prune_percent as usize / 100;
        for _ in 0..to_prune {
            match self.order.pop_front() {
                Some(old) => {
                    self.seen.remove(&old);
                }
                None => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_replays() {
        let mut n = Nonce::default();
        assert!(n.continue_on(1));
        assert!(n.continue_on(2));
        assert!(!n.continue_on(1));
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn prunes_oldest() {
        let mut n = Nonce::new(4, 50); // prune 2 when size exceeds 4
        for id in 0..5 {
            n.continue_on(id);
        }
        // pushing the 5th triggers prune of 2 oldest on the *next* call
        n.continue_on(100);
        assert!(n.continue_on(0)); // 0 was pruned, so it's "new" again
    }
}
