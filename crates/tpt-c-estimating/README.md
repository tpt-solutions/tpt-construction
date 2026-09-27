# tpt-c-estimating

Construction estimating: turning a quantity takeoff into a priced estimate and
a bid. The central type is `EstimateBuilder`, which consumes a `TakeoffResult`
(from `tpt-c-quantities`) and a `CostDatabase` (from `tpt-c-cost`), prices each
measured quantity by its cost code, and assembles an `Estimate`. The crate also
covers bid preparation, versioned estimate revisions with comparison, and a
cost-plan rollup by code. Reach for it whenever measured quantities need unit
rates, markup, and a priced, comparable total.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `EstimateBuilder`: price a takeoff by cost code, with a pluggable `CostDatabase`
- Builder options: `with_database`, `with_markup` (default overhead 10% / profit 8%), `price_gross` vs net, `for_project`
- `from_project` (runs a default takeoff for you) or `from_takeoff` (explicit input)
- `PricingError` distinguishes missing database, missing rate per code, and currency mismatch
- `BidPreparation`: wraps an estimate with bid title, preparer audit meta, and notes
- `EstimateRevision` / `EstimateRevision::compare`: labelled revisions with total, line-count, and per-cost-code deltas
- `CostPlan::from_estimate`: by-code budget rollup (e.g. per MasterFormat division)
- Helpers `code_for` (classification → `CostCode`) and `default_markup`

## Usage

```toml
[dependencies]
tpt-c-estimating = "0.1"
```

```rust
use tpt_c_cost::{CostDatabase, Money, RateUnit, ResourceKind, ResourceRate};
use tpt_c_estimating::{CostPlan, EstimateBuilder, EstimateRevision};
use tpt_c_quantities::TakeoffEngine;

// `project` is a tpt-c-model::Project whose elements carry classification and
// quantity sets (see this crate's tests for the full setup).
let takeoff = TakeoffEngine::new().run(&project);

let mut db = CostDatabase::new();
db.insert(
    "03 30 00",
    ResourceRate::new(
        "r1",
        ResourceKind::Material,
        RateUnit::Volume,
        Money::new(120.0, "USD"),
    )
    .with_description("Concrete CY"),
);

let estimate = EstimateBuilder::new("Demo Estimate", "USD")
    .with_database(db)
    .from_takeoff(&takeoff)?;

let totals = estimate.totals()?; // subtotal plus markup
let plan = CostPlan::from_estimate(&estimate);

let rev_a = EstimateRevision::new(estimate, "baseline");
let rev_total = rev_a.total();
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`, `tpt-c-ids`, `tpt-c-classification`, `tpt-c-model`, `tpt-c-units`, `tpt-c-quantities`, `tpt-c-cost`, `tpt-c-csv` (dev: `serde_json`, `tpt-c-xlsx`)
- **Used by:** `tpt-c-xlsx` (estimate export), the `tpt` CLI (`examples/tpt`), and the `estimate-export` example

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
