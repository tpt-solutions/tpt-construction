# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `LwwTimestamp` (counter + actor tie-breaker) and `LwwRegister<T>` with
  deterministic `merge` (greater timestamp wins, ties broken by actor).
- `GSet<T>` grow-only set with `insert`, `contains`, `len`, `is_empty`,
  `members`, and union `merge`.
- `OrSet<T>` add-wins observed-remove set with `OrTag` tags, `add`, observed
  `remove`, `contains`, `values`, and convergent `merge`.
- `Replica<K, V>` local-first store with `new`, `actor`, `is_online`,
  `go_offline` / `go_online`, `get`, `keys`, `local_put`, `pending_ops`, and
  `take_pending`.
- Two-way `sync(a, b)` free function with `SyncReport` (`applied_to_a`,
  `applied_to_b`, `converged_keys`) and `SyncOp` operation type.
- `SyncError` (`Decode`).
