# tpt-c-change

Change orders, revisions, and cost change control. Construction work rarely
matches the original contract exactly; this crate models the workflow that
absorbs the difference. `ChangeOrder` tracks a proposed change (cost and time
impact) through a review state machine (`Draft → Submitted → UnderReview →
Approved/Rejected`, ending in `Implemented` or `Withdrawn`), `RevisionDiff`
compares two revisions of the same scope into per-key quantity deltas and
per-cost-code cost deltas, and `ChangeRegister` is the per-project ledger with
the approved cost/time roll-ups a project-controls report needs. Reach for it
when contract changes need lifecycle, auditability, and impact arithmetic.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `ChangeOrder` with builder-style `with_description`, `with_cost_impact` (a `Money`, credits negative), `with_time_impact` (days), and costed `ChangeLine`s
- `ChangeOrderStatus` state machine: `transition` validates every move; `is_terminal` / `is_approved` helpers; skips (e.g. `Draft → Approved`) are rejected
- `RevisionDiff::compute`: quantity takeoffs and priced cost codes in, ordered deltas out
- `QuantityDiff` / `QuantityDelta` with `DeltaKind` (`Added`, `Removed`, `Increased`, `Decreased`, `Unchanged`), signed `delta()`, and `total_absolute_delta` churn
- `CostDelta::delta` and `RevisionDiff::net_cost_delta` delegate to `tpt-c-cost::Money`, so currency mismatches are rejected instead of silently combined
- `ChangeRegister`: unique id and number, `get`/`get_mut`/`transition`, `approved_cost_impact`, `approved_time_impact_days`, and `pending()` iteration
- `ChangeError` covers invalid transitions, duplicate numbers, unknown orders, and currency mismatch

## Usage

```toml
[dependencies]
tpt-c-change = "0.1"
```

```rust
use std::collections::BTreeMap;
use tpt_c_change::{ChangeOrder, ChangeOrderStatus, ChangeRegister, QuantityEntry, RevisionDiff};
use tpt_c_core::{ChangeOrderId, ProjectId};
use tpt_c_cost::Money;

let mut reg = ChangeRegister::new(ProjectId::nil());
let mut co = ChangeOrder::new(ChangeOrderId::nil(), "CO-1", "Slab thickening")
    .with_description("Owner added a mezzanine load.")
    .with_cost_impact(Money::new(18_500.0, "USD"))
    .with_time_impact(3.0);
co.transition(ChangeOrderStatus::Submitted)?;
co.transition(ChangeOrderStatus::UnderReview)?;
co.transition(ChangeOrderStatus::Approved)?;
reg.add(co)?;
assert_eq!(reg.approved_cost_impact()?.amount(), 18_500.0);
assert_eq!(reg.approved_time_impact_days(), 3.0);

// 1000 CY -> 1050 CY is a +50 CY delta.
let mut a = BTreeMap::new();
a.insert("concrete".to_string(), QuantityEntry::new("CY", 1000.0));
let mut b = BTreeMap::new();
b.insert("concrete".to_string(), QuantityEntry::new("CY", 1050.0));
let diff = RevisionDiff::compute("rev-A", "rev-B", &a, &b, &BTreeMap::new(), &BTreeMap::new());
assert_eq!(diff.quantities.deltas[0].delta(), 50.0);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core`, `tpt-c-ids`, `tpt-c-cost`
- **Used by:** Not yet consumed by other workspace crates; see the examples/ directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
