# tpt-c-core

Core domain primitives for the tpt-construction workspace: the shared
vocabulary that every other `tpt-c-*` crate is built on. It provides
strongly-typed, UUID-backed identifier newtypes for the main construction
aggregates (projects, contracts, estimates, schedules, ...), the shared error
type, audit metadata captured for every audited mutation, and the project
context plus capability traits that domain aggregates implement.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- 20 UUID-backed id newtypes (`ProjectId`, `ContractId`, `ModelId`,
  `ElementId`, `EstimateId`, `ScheduleId`, `ActivityId`, `RFIId`,
  `SubmittalId`, `ChangeOrderId`, `AssetId`, `ClaimId`, `ContractItemId`,
  `DocumentId`, `IssueId`, `NoticeId`, `PaymentApplicationId`, `PunchListId`,
  `SafetyIncidentId`, `TransmittalId`), each de/serializing transparently,
  ordering deterministically, and parsing via `FromStr`
- `CoreError`, the shared domain error enum (`InvalidId`, `MissingRequired`,
  `InvariantViolated`, `NotFound`, `OutOfRange`, `IllegalState`), with the
  matching `Result<T>` alias
- `AuditMeta` (actor, RFC 3339 timestamp, optional note) with a builder API
- `ProjectContext` describing the project a domain operation runs under:
  name, optional organization, and an ISO 4217 currency code (default `USD`)
- `ProjectRef`, a lightweight reference to the owning project
- `Identified` and `ProjectScoped` traits implemented by domain aggregates
- Pure data and traits only: no IO, and dependencies limited to `serde`,
  `thiserror`, and `uuid`

## Usage

```toml
[dependencies]
tpt-c-core = "0.1"
```

```rust
use tpt_c_core::{AuditMeta, ElementId, ProjectContext, ProjectId};

// Identifiers round-trip through their canonical UUID string.
let id = ProjectId::nil();
assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
let parsed: ElementId = "00000000-0000-0000-0000-000000000000".parse().unwrap();
assert_eq!(parsed.to_string(), id.to_string());

// Audit metadata with a builder-style note.
let audit = AuditMeta::new("alice", "2026-01-01T00:00:00Z").with_note("created");
assert_eq!(audit.actor, "alice");
assert_eq!(audit.note.as_deref(), Some("created"));

// Project context with an explicit currency.
let project = ProjectContext::new("Demo").with_currency("EUR");
assert_eq!(project.currency, "EUR");
assert!(project.organization.is_none());
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid` — no other workspace crates;
  this is the base of the workspace dependency graph
- **Used by:** nearly every workspace crate, including `tpt-c-ids`,
  `tpt-c-model`, `tpt-c-geometry`, `tpt-c-ifc`, `tpt-c-schedule`,
  `tpt-c-estimating`, `tpt-c-cost`, `tpt-c-contracts`, `tpt-c-documents`,
  `tpt-c-workflow`, and `tpt-c-events`, plus the `tpt` CLI and the
  `cpm-schedule` / `field-*` examples

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
