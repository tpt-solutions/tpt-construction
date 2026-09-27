# tpt-c-maintenance

Preventive and corrective maintenance modelling: work orders targeting assets,
their lifecycle status, and an append-only service history. Reach for it when
you need to schedule or record maintenance work (filter changes, repairs,
inspections, modifications) against assets from `tpt-c-assets`, and keep a
read-only trail of who performed each service step.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `WorkOrder`: identifies an `AssetId` target with a string id, maintenance
  kind, status, description and accumulated `ServiceEntry` history.
- `MaintenanceKind` enum: `Preventive`, `Corrective`, `Inspection`,
  `Modification` (serialized as snake_case).
- `MaintenanceStatus` enum: `Requested`, `Scheduled`, `InProgress`,
  `Completed`, `Cancelled`.
- `WorkOrder::transition` advances the status;
  `WorkOrder::add_entry` appends service history.
- `ServiceEntry` records an ISO 8601 date, who performed the work and
  optional notes (`with_notes` builder).
- Serde `Serialize`/`Deserialize` support on all public types.

## Usage

```toml
[dependencies]
tpt-c-maintenance = "0.1"
```

```rust
use tpt_c_ids::IdFactory;
use tpt_c_maintenance::{MaintenanceKind, MaintenanceStatus, ServiceEntry, WorkOrder};

let mut wo = WorkOrder::new(
    "WO-1",
    IdFactory::asset(),
    MaintenanceKind::Preventive,
    "Filter change",
);
assert_eq!(wo.status, MaintenanceStatus::Requested);
wo.transition(MaintenanceStatus::InProgress);
wo.add_entry(ServiceEntry::new("2026-03-01", "alice").with_notes("Replaced filter"));
wo.transition(MaintenanceStatus::Completed);
assert_eq!(wo.history.len(), 1);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-core` (`AssetId`), `tpt-c-ids` (test id factory), `tpt-c-assets` (asset domain).
- **Used by:** Not yet consumed by other workspace crates; see the examples/ directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
