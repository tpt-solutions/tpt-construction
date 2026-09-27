# tpt-c-space

Space inventory for buildings and sites: spaces with categories, net/gross
areas, floors and occupancy counts, plus leases and project-level totals.
This is the space model referenced by the FM and digital twin crates.
Reach for it when you need a serializable registry of rooms/zones with
areas in square metres and the lease records that lettable area is split into.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Space`: id, name, category (office, corridor, mechanical, ...), net area
  in m², and optional gross area, floor and occupancy count.
- Builder methods `with_gross_area`, `with_floor` and `with_occupancy`.
- `Lease`: tenant, ISO 8601 start/end dates and net lettable area.
- `SpaceInventory`: a project-scoped collection (`ProjectId`) of spaces and
  leases, with `add_space`/`add_lease`.
- Roll-up helpers `total_net_area` and `total_leased_area`.
- `Default` inventory uses `ProjectId::nil()`; all public types derive
  `Serialize`/`Deserialize`.

## Usage

```toml
[dependencies]
tpt-c-space = "0.1"
```

```rust
use tpt_c_core::ProjectId;
use tpt_c_space::{Lease, Space, SpaceInventory};

let mut inv = SpaceInventory::new(ProjectId::nil());
inv.add_space(Space::new("S1", "Office A", "office", 25.0).with_gross_area(30.0));
inv.add_space(Space::new("S2", "Office B", "office", 35.0));
inv.add_lease(Lease::new("L1", "Acme Corp", "2026-01-01", "2027-01-01", 25.0));

assert_eq!(inv.total_net_area(), 60.0);
assert_eq!(inv.total_leased_area(), 25.0);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-core` (`ProjectId`), `tpt-c-model`.
- **Used by:** Not yet consumed by other workspace crates; see the examples/ directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
