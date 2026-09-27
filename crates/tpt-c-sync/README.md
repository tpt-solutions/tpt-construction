# tpt-c-sync

Offline-first synchronization built on conflict-free replicated data types
(CRDTs). Field devices often operate with intermittent connectivity; `Replica` is
a local-first store where writes always succeed locally and are queued for later
propagation. When two replicas meet, `sync` exchanges their pending operations
and merges CRDT state deterministically — the same edits converging in any order
always produce identical state, with no central coordinator. Three CRDTs are
provided: a last-writer-wins register (`LwwRegister`), a grow-only set (`GSet`),
and an add-wins observed-remove set (`OrSet`).

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `LwwRegister<T>` with `LwwTimestamp` (Lamport-style counter plus actor
  tie-breaker) so merges are deterministic regardless of message ordering.
- `GSet<T>` grow-only set with union merge, `insert`, `contains`, `len`,
  `is_empty`, and `members`.
- `OrSet<T>` add-wins observed-remove set: unique `OrTag` per add, observed-remove
  tombstones, `values()`, and convergent `merge`.
- `Replica<K, V>` local-first store: `go_offline` / `go_online`, `local_put`
  (immediately visible, queued for propagation), `pending_ops` / `take_pending`,
  and `get` / `keys`.
- Two-way `sync(a, b)` free function returning a `SyncReport` (ops applied each
  way, converged key count).
- Only dependencies are `serde` and `thiserror` — usable with any key/value type
  that is `Clone + Eq + Hash` and serializable.

## Usage

```toml
[dependencies]
tpt-c-sync = "0.1"
```

```rust
use tpt_c_sync::{sync, Replica};

// Two field devices start disconnected and both write locally.
let mut device_a: Replica<String, u32> = Replica::new("A");
let mut device_b: Replica<String, u32> = Replica::new("B");

device_a.go_offline();
device_b.go_offline();

device_a.local_put("hd-exc-1".into(), 100);
device_b.local_put("hd-exc-1".into(), 250); // later write wins
device_b.local_put("hd-dozer-1".into(), 80);

assert_eq!(device_a.pending_ops().len(), 1);
assert_eq!(device_b.pending_ops().len(), 2);

// Reconnect and synchronize; both replicas converge deterministically.
device_a.go_online();
device_b.go_online();
let report = sync(&mut device_a, &mut device_b);

assert_eq!(device_a.get(&"hd-exc-1".into()), Some(&250));
assert_eq!(device_b.get(&"hd-exc-1".into()), Some(&250));
assert_eq!(device_a.get(&"hd-dozer-1".into()), Some(&80));
assert!(report.applied_to_a == 2 && report.applied_to_b == 1);
assert!(device_a.pending_ops().is_empty());
assert!(device_b.pending_ops().is_empty());
```

## Crate relationships

- **Depends on:** `serde`, `thiserror` (no `tpt-c-core` dependency — fully
  generic over caller key/value types).
- **Used by:** Not yet consumed by other workspace crates;
  [examples/field-offline-sync](../../examples/field-offline-sync) replicates a
  snapshot of equipment state between two field devices and converges them.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
