// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Offline-first synchronization built on conflict-free replicated data types
//! (CRDTs).
//!
//! Field devices often operate with intermittent connectivity. [`Replica`] is a
//! local-first store: writes always succeed locally and are queued for later
//! propagation. When two replicas meet, [`sync`] exchanges their pending
//! operations and merges CRDT state deterministically, so the same edits
//! converging from any order always produce identical state — no central
//! coordinator required.
//!
//! Three CRDTs are provided: a last-writer-wins register ([`LwwRegister`]), a
//! grow-only set ([`GSet`]) and an add-wins observed-remove set ([`OrSet`]).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use thiserror::Error;

/// Errors arising during synchronization.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SyncError {
    /// A serialized payload could not be decoded.
    #[error("payload decode failed: {0}")]
    Decode(String),
}

/// A Lamport-style timestamp that breaks ties by actor, making merges fully
/// deterministic across replicas regardless of message ordering.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LwwTimestamp {
    /// Monotonic counter from the originating replica.
    pub counter: u64,
    /// Originating replica/actor identity used as a deterministic tie-breaker.
    pub actor: String,
}

impl LwwTimestamp {
    /// Build a timestamp.
    pub fn new(counter: u64, actor: impl Into<String>) -> Self {
        Self {
            counter,
            actor: actor.into(),
        }
    }
}

/// A last-writer-wins register: carries a value plus the timestamp of the write
/// that produced it. Merging two registers keeps the one with the greater
/// timestamp (ties broken by [`LwwTimestamp`] ordering).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwRegister<T> {
    /// The current value.
    pub value: T,
    /// Timestamp of the winning write.
    pub ts: LwwTimestamp,
}

impl<T: Clone> LwwRegister<T> {
    /// Create a register seeded with an initial value and timestamp.
    pub fn new(value: T, ts: LwwTimestamp) -> Self {
        Self { value, ts }
    }

    /// Merge `other` into `self`, keeping the value with the greater timestamp.
    pub fn merge(&mut self, other: &LwwRegister<T>) {
        if other.ts >= self.ts {
            self.value = other.value.clone();
            self.ts = other.ts.clone();
        }
    }
}

/// A grow-only set: an element, once added, is never removed. Merging is set
/// union.
#[derive(Clone, Debug)]
pub struct GSet<T> {
    members: HashSet<T>,
}

impl<T: Clone + Eq + Hash> Default for GSet<T> {
    fn default() -> Self {
        Self {
            members: HashSet::new(),
        }
    }
}

impl<T: Clone + Eq + Hash> GSet<T> {
    /// Create an empty grow-only set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an element.
    pub fn insert(&mut self, value: T) {
        self.members.insert(value);
    }

    /// Whether the set contains `value`.
    pub fn contains(&self, value: &T) -> bool {
        self.members.contains(value)
    }

    /// Number of members.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// All members.
    pub fn members(&self) -> impl Iterator<Item = &T> {
        self.members.iter()
    }

    /// Merge `other` into `self` via set union.
    pub fn merge(&mut self, other: &GSet<T>) {
        for m in &other.members {
            self.members.insert(m.clone());
        }
    }
}

/// A tag used to identify individual adds in an [`OrSet`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrTag(pub String);

/// An add-wins observed-remove set.
///
/// Each `add` mints a unique tag; an element is present if it has at least one
/// tag that has not been observed-removed. `remove` tombstones every tag of the
/// element that was *observed* by the caller (add-wins semantics).
#[derive(Clone, Debug)]
pub struct OrSet<T> {
    /// Live tags per element.
    added: HashMap<T, HashSet<OrTag>>,
    /// Tags that have been removed (tombstones).
    removed: HashSet<OrTag>,
}

impl<T: Clone + Eq + Hash> Default for OrSet<T> {
    fn default() -> Self {
        Self {
            added: HashMap::new(),
            removed: HashSet::new(),
        }
    }
}

impl<T: Clone + Eq + Hash> OrSet<T> {
    /// Create an empty OR-Set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `value`, tagging it with a unique tag derived from `tag_seed`.
    pub fn add(&mut self, value: T, tag_seed: impl Into<String>) {
        let tag = OrTag(tag_seed.into());
        self.removed.remove(&tag);
        self.added.entry(value).or_default().insert(tag);
    }

    /// Remove every tag of `value` currently present (observed remove).
    /// Returns `true` if any tag was tombstoned.
    pub fn remove(&mut self, value: &T) -> bool {
        let mut removed_any = false;
        if let Some(tags) = self.added.get(value) {
            for tag in tags {
                if !self.removed.contains(tag) {
                    self.removed.insert(tag.clone());
                    removed_any = true;
                }
            }
        }
        removed_any
    }

    /// Whether `value` currently has at least one live (non-tombstoned) tag.
    pub fn contains(&self, value: &T) -> bool {
        self.added
            .get(value)
            .map(|tags| tags.iter().any(|t| !self.removed.contains(t)))
            .unwrap_or(false)
    }

    /// All currently-present values.
    pub fn values(&self) -> Vec<T> {
        self.added
            .keys()
            .filter(|v| self.contains(v))
            .cloned()
            .collect()
    }

    /// Merge `other` into `self` (add-wins OR-Set union of adds and removes).
    pub fn merge(&mut self, other: &OrSet<T>) {
        for (value, tags) in &other.added {
            let entry = self.added.entry(value.clone()).or_default();
            for t in tags {
                entry.insert(t.clone());
            }
        }
        for t in &other.removed {
            self.removed.insert(t.clone());
        }
    }
}

/// A pending change queued for propagation to peer replicas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SyncOp<K, V> {
    /// Upsert a keyed CRDT register.
    Put {
        /// Entity key.
        key: K,
        /// Register state to merge.
        register: LwwRegister<V>,
    },
}

/// Summary produced after a two-way synchronization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncReport {
    /// Operations applied into replica A from replica B.
    pub applied_to_a: usize,
    /// Operations applied into replica B from replica A.
    pub applied_to_b: usize,
    /// Keys that both replicas held (and thus converged) during the sync.
    pub converged_keys: usize,
}

/// A local-first replica holding keyed LWW registers with a pending out-queue.
#[derive(Clone, Debug)]
pub struct Replica<K, V> {
    actor: String,
    clock: u64,
    online: bool,
    state: HashMap<K, LwwRegister<V>>,
    pending: Vec<SyncOp<K, V>>,
}

impl<K, V> Replica<K, V>
where
    K: Clone + Eq + Hash + Serialize + for<'de> Deserialize<'de>,
    V: Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a replica identified by `actor`.
    pub fn new(actor: impl Into<String>) -> Self {
        Self {
            actor: actor.into(),
            clock: 0,
            online: true,
            state: HashMap::new(),
            pending: Vec::new(),
        }
    }

    /// The replica's actor identity.
    pub fn actor(&self) -> &str {
        &self.actor
    }

    /// Whether the replica is currently allowing propagation.
    pub fn is_online(&self) -> bool {
        self.online
    }

    /// Mark the replica offline (writes still succeed locally, just queue).
    pub fn go_offline(&mut self) {
        self.online = false;
    }

    /// Mark the replica online.
    pub fn go_online(&mut self) {
        self.online = true;
    }

    /// Read the current value for `key`, if present.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.state.get(key).map(|r| &r.value)
    }

    /// All keys currently held.
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.state.keys()
    }

    /// Locally write `value` for `key`. The write is immediately visible and,
    /// if online, queued for propagation.
    pub fn local_put(&mut self, key: K, value: V) {
        self.clock += 1;
        let ts = LwwTimestamp::new(self.clock, self.actor.clone());
        let register = LwwRegister::new(value, ts);
        self.state.insert(key.clone(), register.clone());
        self.pending.push(SyncOp::Put { key, register });
    }

    /// Pending operations awaiting propagation.
    pub fn pending_ops(&self) -> &[SyncOp<K, V>] {
        &self.pending
    }

    /// Drain the pending out-queue.
    pub fn take_pending(&mut self) -> Vec<SyncOp<K, V>> {
        std::mem::take(&mut self.pending)
    }

    /// Merge a batch of remote operations into local state.
    fn merge_remote(&mut self, ops: Vec<SyncOp<K, V>>) {
        for op in ops {
            let SyncOp::Put { key, register } = op;
            self.clock = self.clock.max(register.ts.counter);
            match self.state.get_mut(&key) {
                Some(existing) => existing.merge(&register),
                None => {
                    self.state.insert(key, register);
                }
            }
        }
    }
}

/// Perform a two-way synchronization between two replicas.
///
/// Each replica's pending operations are exchanged and merged into the other.
/// Because LWW registers merge deterministically, both replicas converge to the
/// identical state regardless of which edits happened on which device.
pub fn sync<K, V>(a: &mut Replica<K, V>, b: &mut Replica<K, V>) -> SyncReport
where
    K: Clone + Eq + Hash + Serialize + for<'de> Deserialize<'de>,
    V: Clone + PartialEq + Serialize + for<'de> Deserialize<'de>,
{
    let ops_a = a.take_pending();
    let ops_b = b.take_pending();
    let applied_to_b = ops_a.len();
    let applied_to_a = ops_b.len();
    a.merge_remote(ops_b);
    b.merge_remote(ops_a);

    let mut converged = 0;
    for key in a.state.keys() {
        if b.state.contains_key(key) {
            converged += 1;
        }
    }
    SyncReport {
        applied_to_a,
        applied_to_b,
        converged_keys: converged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lww_register_picks_later_write() {
        let mut a = LwwRegister::new("v1", LwwTimestamp::new(1, "a"));
        let b = LwwRegister::new("v2", LwwTimestamp::new(2, "a"));
        a.merge(&b);
        assert_eq!(a.value, "v2");
    }

    #[test]
    fn lww_tie_broken_by_actor() {
        let mut a = LwwRegister::new("a-wins", LwwTimestamp::new(1, "a"));
        let b = LwwRegister::new("b-loses", LwwTimestamp::new(1, "b"));
        // "a" < "b", so b's timestamp is greater.
        a.merge(&b);
        assert_eq!(a.value, "b-loses");
    }

    #[test]
    fn gset_union() {
        let mut a: GSet<u32> = GSet::new();
        let mut b: GSet<u32> = GSet::new();
        a.insert(1);
        b.insert(2);
        a.merge(&b);
        assert!(a.contains(&1) && a.contains(&2));
    }

    #[test]
    fn orset_add_wins() {
        let mut a: OrSet<String> = OrSet::new();
        a.add("x".into(), "t1");
        let mut b = a.clone();
        b.remove(&"x".into());
        assert!(!b.contains(&"x".into()));
        // concurrent re-add on a wins over b's remove
        a.add("x".into(), "t2");
        a.merge(&b);
        b.merge(&a);
        assert!(a.contains(&"x".into()));
        assert!(b.contains(&"x".into()));
    }

    #[test]
    fn offline_writes_then_sync_converge() {
        let mut device_a: Replica<String, u32> = Replica::new("A");
        let mut device_b: Replica<String, u32> = Replica::new("B");

        device_a.go_offline();
        device_b.go_offline();

        device_a.local_put("hd-exc-1".into(), 100);
        device_b.local_put("hd-exc-1".into(), 250);
        device_b.local_put("hd-dozer-1".into(), 80);

        assert_eq!(device_a.pending_ops().len(), 1);
        assert_eq!(device_b.pending_ops().len(), 2);

        device_a.go_online();
        device_b.go_online();
        let report = sync(&mut device_a, &mut device_b);

        // Both replicas converge to B's later write for the shared key.
        assert_eq!(device_a.get(&"hd-exc-1".into()), Some(&250));
        assert_eq!(device_b.get(&"hd-exc-1".into()), Some(&250));
        assert_eq!(device_a.get(&"hd-dozer-1".into()), Some(&80));

        assert_eq!(report.applied_to_a, 2);
        assert_eq!(report.applied_to_b, 1);
        assert!(report.converged_keys >= 1);
        assert!(device_a.pending_ops().is_empty());
        assert!(device_b.pending_ops().is_empty());
    }
}
