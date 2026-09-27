# 3. Price an estimate from a CSV cost database

> **Crates used:** `tpt-c-model`, `tpt-c-estimating`, `tpt-c-csv`, `tpt-c-quantities`, `tpt-c-ids`, `tpt-c-units`

The estimating crate turns takeoff quantities into a priced estimate by
looking up a rate for every quantity's cost code. Codes come from an
element's classification when it has one, or from its category when it
doesn't. This recipe loads rates from the CSV format the `tpt` CLI uses.

## Cargo.toml

```toml
[dependencies]
tpt-c-core = "0.1"
tpt-c-csv = "0.1"
tpt-c-estimating = "0.1"
tpt-c-ids = "0.1"
tpt-c-model = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use tpt_c_core::ProjectId;
use tpt_c_csv::CostRateRow;
use tpt_c_estimating::EstimateBuilder;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, Quantity, QuantitySet};
use tpt_c_units::Volume;

fn main() {
    // One slab, one category. With no classification attached the category
    // string IS the cost code the estimator must price.
    let mut project = Project::new(ProjectId::nil(), "Riverside Outbuilding");
    project.add_element(
        Element::new(
            IdFactory::deterministic_element("slab-1"),
            "SLAB-1",
            "Slab",
        )
        .with_quantity_set(
            QuantitySet::new("BaseQuantities").with(
                "GrossVolume",
                Quantity::Volume(Volume::from_cubic_meters(9.6)),
            ),
        ),
    );

    // A rate database in the same CSV dialect as
    // test-data/golden/rates.csv (header: code,title,kind,unit,rate,currency).
    let csv = "code,title,kind,unit,rate,currency\n\
               Slab,Slab-on-grade package,material,volume,185,USD\n";
    let db = tpt_c_csv::read_cost_database(csv.as_bytes()).unwrap();

    // Model -> takeoff -> line items -> estimate. Any quantity whose code is
    // missing from the database is an error, not a silent zero.
    let takeoff = tpt_c_quantities::TakeoffEngine::new().run(&project);
    let estimate = EstimateBuilder::new("Riverside Estimate", "USD")
        .with_database(db)
        .from_takeoff(&takeoff)
        .unwrap();

    let totals = estimate.totals().unwrap();
    println!(
        "{} line item(s); subtotal {:.2} {}",
        estimate.line_items.len(),
        totals.subtotal.amount(),
        estimate.currency,
    );
    println!(
        "with markup, taxes and escalation: total {:.2} {}",
        totals.total.amount(),
        estimate.currency,
    );
    let _ = CostRateRow::header(); // the CSV dialect's header row
}
```

## Run it

```bash
cargo run
```

Output (markup etc. from the default design-stage markup schedule):

```text
1 line item(s); subtotal 1900.32 USD
with markup, taxes and escalation: total 2257.58 USD
```

## How it works

1. `read_cost_database` parses the CSV into a `CostDatabase` — a code → rate
   map with currency-aware `Money` values. Any `R: std::io::Read` works, so
   files, byte cursors, and network streams are interchangeable.
2. `EstimateBuilder::new(title, currency)` fixes the estimate currency; rates
   in any other currency are rejected rather than converted.
3. `from_takeoff` maps each measured quantity to a cost code
   (`tpt_c_estimating::code_for`): classification first, category fallback —
   then multiplies the (gross) quantity by the rate and appends a `LineItem`.
4. `totals()` returns subtotal plus the applied markup/tax/escalation
   breakdown. Attach a classification such as MasterFormat `03 30 00` via
   `Element::classified` to price against an industry code list instead
   (see `test-data/golden/` for a full worked example).
