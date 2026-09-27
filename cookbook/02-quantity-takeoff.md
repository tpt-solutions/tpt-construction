# 2. Quantity takeoff

> **Crates used:** `tpt-c-model`, `tpt-c-quantities`, `tpt-c-ids`, `tpt-c-units`

The takeoff engine reads a neutral model and produces net/gross quantities
per element. Explicit `QuantitySet` values flow through unchanged; derived
rules (concrete volume from layers, formwork area, rebar weight, paint area)
fill in what geometry allows, and waste factors expand net into gross.

## Cargo.toml

```toml
[dependencies]
tpt-c-core = "0.1"
tpt-c-ids = "0.1"
tpt-c-model = "0.1"
tpt-c-quantities = "0.1"
tpt-c-units = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use tpt_c_core::ProjectId;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, Quantity, QuantitySet};
use tpt_c_quantities::{QuantityKind, TakeoffEngine};
use tpt_c_units::Volume;

fn main() {
    // A two-element model: the slab stores its volume explicitly.
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

    // Run the takeoff. Rules are configurable via TakeoffEngine::with_rules;
    // the defaults apply trade-typical waste factors by category.
    let result = TakeoffEngine::new().run(&project);

    println!(
        "takeoff for '{}': {} elements",
        project.name,
        result.items.len()
    );
    for item in &result.items {
        println!("[{}] {} ({})", item.element_id, item.name, item.category);
        for q in &item.quantities {
            println!(
                "  {:<15} net {:8.3}  gross {:8.3}  waste {:4.1}%{}",
                q.name,
                q.net.base_value(),
                q.gross().base_value(),
                q.waste.ratio() * 100.0,
                if q.manual { "  [override]" } else { "" },
            );
        }
    }

    // Roll-ups by quantity kind, in canonical base units (m3 for volume).
    println!(
        "total volume: net {:.3} m3, gross {:.3} m3",
        result.total_net(QuantityKind::Volume),
        result.total_gross(QuantityKind::Volume),
    );
}
```

## Run it

```bash
cargo run
```

Output (element id is deterministic):

```text
takeoff for 'Riverside Outbuilding': 1 elements
[c86885dd-4b46-51a2-922b-bbb2227721d4] SLAB-1 (Slab)
  GrossVolume     net    9.600  gross   10.272  waste  7.0%
total volume: net 9.600 m3, gross 10.272 m3
```

## How it works

1. `TakeoffEngine::run` visits every element and collects quantities from its
   `QuantitySet`s (marked `manual: false` — set `manual: true` via a manual
   override when a quantity was hand-adjusted).
2. Derived rules only fire when the element has **no** stored quantity of
   that kind: a stored `Volume` suppresses the concrete-volume rule, missing
   areas invite the formwork/paint rules, and so on. That makes explicit
   model data authoritative while still pricing dumb geometry.
3. `q.net` is a typed measure (`Volume`, `Area`, …); `base_value()` returns
   the canonical SI magnitude so totals are unit-safe.
4. `gross()` expands net by the waste factor chosen for the element's
   category — the number estimators actually order against.
