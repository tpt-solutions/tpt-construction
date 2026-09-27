# tpt-c-contracts

Contracts and contract administration. A `Contract` groups the parties, line
`ContractItem`s, and assigned responsibilities, and carries the administrative
events that surround it: formal `Notice`s (driven by `tpt-c-workflow`'s
`NoticeWorkflow`), `Claim`s (delay, disruption, acceleration, payment), and
`ComplianceEvent`s such as permits and obligations with due dates. Amounts are
kept as `(f64, currency)` pairs so the crate stays free of the cost-model
dependency.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Contract` aggregate: `add_party`, `add_responsibility`, `add_item`, and
  `total_value()` which sums item `Amount`s and rejects currency mismatches.
- `Amount`: a monetary value plus ISO 4217 currency code, with `checked_add`
  enforcing a single currency.
- `Notice` backed by a `NoticeWorkflow` with convenience `acknowledge` / `resolve`
  methods and `state()` access.
- `Claim` with `ClaimKind` (delay, disruption, acceleration, payment, other),
  optional claimed `Amount`, and a `submit` / `resolve` status lifecycle.
- `ComplianceEvent` with `ComplianceStatus` (open, satisfied, overdue) for permits,
  inspections, and contractual obligations.
- `ContractError` for not-found and out-of-range conditions.
- Serializable with serde; a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-contracts = "0.1"
```

```rust
use tpt_c_contracts::{Amount, Claim, ClaimKind, ComplianceEvent, Contract, Notice};
use tpt_c_core::AuditMeta;
use tpt_c_ids::{ContractId, NoticeId};
use uuid::Uuid;

let at = |who: &str| AuditMeta::new(who, "2026-01-01T00:00:00Z");

// A contract with priced line items.
let mut c = Contract::new(ContractId::from_uuid(Uuid::now_v7()), "GC Agreement");
c.add_item("Concrete", Amount::new(100_000.0, "USD"));
c.add_item("Steel", Amount::new(250_000.0, "USD"));
assert_eq!(c.total_value().unwrap().value, 350_000.0);

// A notice whose workflow drives its state.
let mut n = Notice::new(NoticeId::from_uuid(Uuid::now_v7()), "Notice of delay");
n.acknowledge(at("owner"));
assert_eq!(n.state(), tpt_c_workflow::NoticeState::Acknowledged);

// Claims and compliance obligations.
let mut claim = Claim::new(
    tpt_c_ids::ClaimId::from_uuid(Uuid::now_v7()),
    ClaimKind::Delay,
    "Weather delay",
);
claim.submit();
let mut permit = ComplianceEvent::new("PERMIT-1", "Building permit", "2026-02-01");
permit.satisfy();
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core`, `tpt-c-ids`
  (`ContractId`, `ContractItemId`, `NoticeId`, `ClaimId`), and `tpt-c-workflow`
  (`NoticeWorkflow`, `NoticeState`).
- **Used by:** Not yet consumed by other workspace crates;
  [examples/field-execution-e2e](../../examples/field-execution-e2e) builds a
  contract with items, a notice, and a claim in its scenario.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
