# tpt-c-model

The neutral construction domain model. File-format parsers (IFC, glTF, ...)
and domain engines (takeoff, cost, scheduling) all speak this model: a
`Project` aggregate holding sites, buildings, storeys, assemblies, zones,
systems, and a flat registry of `Element`s with property sets, quantity
sets, material layers, classifications, and external ids. It is
serializable, deterministic, and free of any IO; geometry is referenced
opaquely, with rich geometric processing left to `tpt-c-geometry`.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Project` aggregate root with `add_element`, `element`, `elements_of_category`,
  `element_count`, and a configurable default classification system
- `Element` builder (`in_storey`, `classified`, `with_external_id`,
  `with_layer`, `with_property_set`, `with_quantity_set`) implementing the
  `Identified` trait from `tpt-c-core`
- Typed `PropertyValue` (`Text` / `Number` / `Boolean`) grouped into named
  `PropertySet`s
- `Quantity` (`Length` / `Area` / `Volume` / `Count` / `Mass` / `Duration`)
  with `base_value()` in canonical SI units, grouped into `QuantitySet`s
- `MaterialLayer` stacks for layered elements (walls, slabs, roofs)
- Spatial containers `Site`, `Building`, `Storey` and groupings `Assembly`,
  `Zone`, `System`
- Fully `serde`-serializable and deterministic; consumes `tpt-c-core`,
  `tpt-c-ids`, `tpt-c-units`, and `tpt-c-classification` types

## Usage

```toml
[dependencies]
tpt-c-model = "0.1"
```

```rust
use tpt_c_classification::{Classification, ClassificationSystem};
use tpt_c_core::ProjectId;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, Quantity, QuantitySet};
use tpt_c_units::Length;

let mut p = Project::new(ProjectId::nil(), "Demo")
    .with_default_system(ClassificationSystem::UniFormat);
let e = Element::new(IdFactory::element(), "W1", "Wall")
    .classified(Classification::new(
        ClassificationSystem::MasterFormat,
        "03 30 00",
    ))
    .with_quantity_set(
        QuantitySet::new("BaseQuantities")
            .with("Length", Quantity::Length(Length::from_feet(20.0))),
    );
p.add_element(e);

assert_eq!(p.element_count(), 1);
assert_eq!(p.elements_of_category("Wall").count(), 1);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-classification`, `tpt-c-core`,
  `tpt-c-ids`, `tpt-c-units`
- **Used by:** `tpt-c-ifc`, `tpt-c-gltf`, `tpt-c-cost`, `tpt-c-estimating`,
  `tpt-c-quantities`, `tpt-c-wasm`, and `tpt-c-xlsx`, plus the `tpt` CLI and
  the `ifc-import`, `estimate-export`, `quantity-takeoff`, and
  `wasm-browser-demo` examples

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
