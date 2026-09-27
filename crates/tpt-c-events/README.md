# tpt-c-events

Domain events, event sourcing, projections and replay. Every meaningful mutation
in the TPT domain is captured as a `DomainEvent` wrapped in a `StoredEvent`
envelope recording a monotonic sequence number, a unique event id, the actor, and
an RFC 3339 timestamp. An `EventStore` is an append-only, content-addressed log
that doubles as the system of record and the audit trail; `Projection`s derive
read models by folding over the log and can be rebuilt at any time via `replay`.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- Nine typed `DomainEvent` variants spanning the domain (`ModelImported`,
  `QuantityAdjusted`, `CostItemApplied`, `EstimateApproved`, `ScheduleUpdated`,
  `RFICreated`, `RFIAnswered`, `SubmittalApproved`,
  `PaymentApplicationSubmitted`), each with a stable `kind()` name.
- `EventStore<E>`: append-only log with `append` (assigns 1-based sequence),
  `version`, `get(seq)`, `all`, `iter_from(offset)` for incremental replay, and
  `to_json` / `from_json` persistence of the whole log.
- `Projection<E>` trait (associated `State` + `fold`) with free functions
  `replay` (full rebuild) and `replay_from` (incremental refresh).
- Batteries-included projections: `EventTypeCounter` (counts per event kind) and
  `AuditTimeline` (chronological `AuditLine`s); `AuditTrail` alias for the
  canonical `EventStore<DomainEvent>` audit log.
- Serializable with serde; a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-events = "0.1"
```

```rust
use std::collections::HashMap;
use tpt_c_core::{ModelId, RFIId};
use tpt_c_events::{replay, AuditTrail, DomainEvent, EventStore, EventTypeCounter};
use tpt_c_ids::IdFactory;

let mut audit: AuditTrail = EventStore::new();
audit.append(
    DomainEvent::ModelImported {
        model_id: ModelId::from_uuid(IdFactory::deterministic("m1")),
        element_count: 42,
    },
    "importer",
    "2026-01-01T00:00:00Z",
);
audit.append(
    DomainEvent::EstimateApproved {
        estimate_id: IdFactory::estimate(),
        by: "chief-estimator".into(),
    },
    "chief-estimator",
    "2026-01-02T00:00:00Z",
);
audit.append(
    DomainEvent::RFICreated {
        rfi_id: RFIId::from_uuid(IdFactory::deterministic("rfi1")),
        subject: "Rebar lap splice".into(),
    },
    "field-eng",
    "2026-01-03T00:00:00Z",
);

assert_eq!(audit.version(), 3);
assert_eq!(audit.get(1).unwrap().seq, 1);

let counts = replay(&audit, &EventTypeCounter, HashMap::new());
assert_eq!(counts.get("model_imported"), Some(&1));
assert_eq!(counts.get("rfi_created"), Some(&1));
```

## Crate relationships

- **Depends on:** `serde`, `serde_json`, `uuid`, `tpt-c-core` (typed ids such as
  `ModelId`, `EstimateId`, `RFIId`), and `tpt-c-ids` (`IdFactory`).
- **Used by:** Not yet consumed by other workspace crates;
  [examples/field-offline-sync](../../examples/field-offline-sync) appends field
  events to a local `AuditTrail` and replays them into an `EventTypeCounter`.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
