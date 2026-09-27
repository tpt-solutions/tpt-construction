# tpt-c-quantities

Quantity takeoff engine for construction estimating. A *takeoff* converts a
neutral [`tpt-c-model`](../tpt-c-model) `Project` into measurable quantities
(count, length, area, volume, mass). Each quantity is tracked as a **net** value
(taken from the model) and a **gross** value (net expanded by a waste factor),
and model-derived values can be locally overridden by an estimator. A derived
rule set (`rules`) computes quantities a parser may not have extracted directly —
concrete volume, formwork area, rebar weight, paint area, and flooring area with
deductions.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `TakeoffEngine::run(&Project) -> TakeoffResult` — per-element quantity extraction
- Net vs gross quantities: `TakeoffQuantity::gross()` applies the waste factor
- Derived rules: `concrete_volume`, `formwork_area`, `rebar_weight_for` (US bar
  sizes #3–#11), `paint_area`, `flooring_area_with_deductions`
- Derived quantities are only added when no stored quantity of that kind exists
  (no duplicate volumes/areas)
- `TakeoffResult::override_quantity` — estimator overrides flagged `manual`
- `total_net(kind)` / `total_gross(kind)` roll-ups by `QuantityKind`
- `QuantityRules` — per-category waste overrides over documented
  `DEFAULT_WASTE_*` defaults
- `MeasuredQuantity` conversions to/from `tpt_c_model::Quantity`

## Usage

```toml
[dependencies]
tpt-c-quantities = "0.1"
```

```rust
use tpt_c_core::ProjectId;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, Quantity, QuantitySet};
use tpt_c_quantities::{MeasuredQuantity, QuantityKind, TakeoffEngine};
use tpt_c_units::Volume;

let mut project = Project::new(ProjectId::nil(), "Demo");
let slab = IdFactory::element();
project.add_element(Element::new(slab, "S1", "Slab").with_quantity_set(
    QuantitySet::new("Q")
        .with("GrossVolume", Quantity::Volume(Volume::from_cubic_yards(2.0))),
));

// Extract quantities with default waste rules.
let mut result = TakeoffEngine::new().run(&project);
assert!(result.total_gross(QuantityKind::Volume) > result.total_net(QuantityKind::Volume));

// Estimator override: replaces the model value and flags it `manual`.
result.override_quantity(
    slab,
    "GrossVolume",
    MeasuredQuantity::Volume(Volume::from_cubic_yards(9.0)),
);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`, `tpt-c-classification`,
  `tpt-c-model`, `tpt-c-units`
- **Used by:** [`tpt-c-estimating`](../tpt-c-estimating) (estimate builder),
  [`tpt-c-wasm`](../tpt-c-wasm), `examples/quantity-takeoff`,
  `examples/wasm-browser-demo`, and the `examples/tpt` web app

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
