# 4. Export an estimate to Excel

> **Crates used:** `tpt-c-estimating`, `tpt-c-xlsx`, plus the model chain from recipe 3

`tpt-c-xlsx` writes real `.xlsx` workbooks (hand-rolled OOXML — no heavy
spreadsheet dependencies). `write_estimate` produces the two-sheet workbook
the `tpt estimate` CLI ships; `write_table` dumps any string table.

## Cargo.toml

```toml
[dependencies]
tpt-c-core = "0.1"
tpt-c-cost = "0.1"
tpt-c-estimating = "0.1"
tpt-c-ids = "0.1"
tpt-c-model = "0.1"
tpt-c-quantities = "0.1"
tpt-c-units = "0.1"
tpt-c-xlsx = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use tpt_c_core::ProjectId;
use tpt_c_estimating::EstimateBuilder;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, Quantity, QuantitySet};
use tpt_c_units::Volume;

fn main() {
    // Model -> takeoff -> estimate (recipe 3 condensed).
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
    let takeoff = tpt_c_quantities::TakeoffEngine::new().run(&project);
    let estimate = EstimateBuilder::new("Riverside Estimate", "USD")
        .with_database(demo_db())
        .from_takeoff(&takeoff)
        .unwrap();

    // Two-sheet workbook: "Estimate" (line items) + "Summary" (markups, total).
    tpt_c_xlsx::write_estimate("estimate.xlsx", &estimate).unwrap();

    // Any tabular snapshot can ride along in its own sheet.
    let headers: Vec<String> = ["item", "qty", "rate", "extension"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let rows: Vec<Vec<String>> = estimate
        .line_items
        .iter()
        .map(|li| {
            vec![
                li.description.clone(),
                format!("{:.2}", li.quantity.base_value()),
                format!("{:.2}", li.unit_rate.amount()),
                format!("{:.2}", li.extension.amount()),
            ]
        })
        .collect();
    tpt_c_xlsx::write_table("line-items.xlsx", "LineItems", &headers, &rows).unwrap();

    println!(
        "wrote estimate.xlsx and line-items.xlsx ({} line items)",
        estimate.line_items.len()
    );
}

fn demo_db() -> tpt_c_cost::CostDatabase {
    let mut db = tpt_c_cost::CostDatabase::new();
    db.insert(
        "Slab",
        tpt_c_cost::ResourceRate::new(
            "r1",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Volume,
            tpt_c_cost::Money::new(185.0, "USD"),
        )
        .with_description("Slab-on-grade package"),
    );
    db
}
```

## Run it

```bash
cargo run
```

```text
wrote estimate.xlsx and line-items.xlsx (1 line items)
```

Open `estimate.xlsx` in Excel/LibreOffice: sheet **Estimate** lists each line
item, sheet **Summary** carries the markup breakdown and final total.

## How it works

1. `write_estimate` serializes the estimate into SpreadsheetML worksheets,
   zips them (`zip` + `flate2`, no bundled compression surprises) and writes
   a valid `.xlsx` — verify with `python -c "import zipfile; zipfile.ZipFile('estimate.xlsx').testzip()"`.
2. `write_table` writes a plain string table; numeric-looking cells are
   emitted as real numbers so Excel can sum them.
3. `LineItem` exposes `description`, `quantity`, `unit`, `rate`, and
   `total()` — everything a bid summary or owner report needs.

> The `demo_db` helper exists only to keep the recipe focused; in production
> you would `read_cost_database` from your priced CSV (recipe 3).
