# 8. Change orders and revision diffs

> **Crates used:** `tpt-c-change`, `tpt-c-cost`, `tpt-c-ids`, `tpt-c-core`

When scope moves, `tpt-c-change` keeps the paperwork and the math honest:
change orders walk a review state machine, revision diffs quantify exactly
what moved (the spec's canonical example: 1000 CY → 1050 CY = +50 CY), and
the register rolls up approved cost/time impact for project-controls
reporting.

## Cargo.toml

```toml
[dependencies]
tpt-c-change = "0.1"
tpt-c-core = "0.1"
tpt-c-cost = "0.1"
tpt-c-ids = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;

use tpt_c_change::{
    ChangeOrder, ChangeOrderStatus, ChangeRegister, QuantityEntry, RevisionDiff,
};
use tpt_c_core::{ChangeOrderId, ProjectId};
use tpt_c_cost::Money;
use tpt_c_ids::IdFactory;

fn main() {
    // --- A change order moves through the review state machine -------------
    let mut co = ChangeOrder::new(
        ChangeOrderId::from_uuid(IdFactory::deterministic("co-1")),
        "CO-1",
        "Slab thickening",
    )
    .with_description("Owner added a mezzanine load; slab requires #5 bars.")
    .with_cost_impact(Money::new(18_500.0, "USD"))
    .with_time_impact(3.0);

    // Illegal jumps are errors: co.transition(Approved) from Draft would fail.
    co.transition(ChangeOrderStatus::Submitted).unwrap();
    co.transition(ChangeOrderStatus::UnderReview).unwrap();
    co.transition(ChangeOrderStatus::Approved).unwrap();

    // --- The register tracks every change order for a project --------------
    let mut register = ChangeRegister::new(ProjectId::nil());
    register.add(co).unwrap(); // duplicate ids/numbers are rejected

    let mut pending = ChangeOrder::new(
        ChangeOrderId::from_uuid(IdFactory::deterministic("co-2")),
        "CO-2",
        "Additional exterior outlet",
    )
    .with_cost_impact(Money::new(1_200.0, "USD"))
    .with_time_impact(0.0);
    pending.transition(ChangeOrderStatus::Submitted).unwrap();
    register.add(pending).unwrap();

    println!("orders: {}", register.orders().len());
    println!(
        "approved cost impact: {:.2} USD, approved time impact: {:.1} days",
        register.approved_cost_impact().unwrap().amount(),
        register.approved_time_impact_days(),
    );
    println!("pending review: {}", register.pending().count());

    // --- Revision diffs: what did the change actually move? ---------------
    // Revision A takeoff (spec §12's example: 1000 CY of concrete).
    let mut rev_a = BTreeMap::new();
    rev_a.insert("concrete".to_string(), QuantityEntry::new("CY", 1000.0));
    // Revision B: same scope, re-measured after the change.
    let mut rev_b = BTreeMap::new();
    rev_b.insert("concrete".to_string(), QuantityEntry::new("CY", 1050.0));
    rev_b.insert("rebar".to_string(), QuantityEntry::new("t", 4.2));

    let mut costs_a = BTreeMap::new();
    costs_a.insert("03 30 00".to_string(), Money::new(120_000.0, "USD"));
    let mut costs_b = BTreeMap::new();
    costs_b.insert("03 30 00".to_string(), Money::new(126_000.0, "USD"));

    let diff = RevisionDiff::compute("rev-A", "rev-B", &rev_a, &rev_b, &costs_a, &costs_b);
    for d in &diff.quantities.deltas {
        println!(
            "{:<10} {:>8.1} -> {:>8.1} {}  ({:+.1})",
            d.key, d.before, d.after, d.unit,
            d.delta(),
        );
    }
    println!(
        "net cost delta: {:+.2} USD",
        diff.net_cost_delta().unwrap().amount(),
    );
}
```

## Run it

```bash
cargo run
```

```text
orders: 2
approved cost impact: 18500.00 USD, approved time impact: 3.0 days
pending review: 1
concrete     1000.0 ->   1050.0 CY  (+50.0)
rebar           0.0 ->      4.2 t  (+4.2)
net cost delta: +6000.00 USD
```

## How it works

1. `ChangeOrderStatus` is a guarded state machine: `Draft → Submitted →
   UnderReview → Approved → Implemented`, with `Rejected`/`Withdrawn`
   terminal states. `transition` returns `ChangeError::InvalidTransition`
   for any jump not in that graph.
2. `ChangeRegister` enforces uniqueness of both id and number, and answers
   the report questions: `approved_cost_impact` (sums only
   approved/implemented orders), `approved_time_impact_days`, and
   `pending()`.
3. `RevisionDiff::compute` diffs two named revisions of quantity maps and
   priced cost codes, producing per-key `QuantityDelta`s (`Added`, `Removed`,
   `Increased`, `Decreased`, `Unchanged`) and per-code `CostDelta`s. Units
   are carried through; `net_cost_delta` rejects mixed currencies.
4. Wire it to the rest of the workspace: run the diff on two
   `tpt-c-estimating` revisions (`EstimateRevision::compare`) and log the
   transitions as `tpt-c-events` domain events for the audit trail.
