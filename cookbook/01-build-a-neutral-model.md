# 1. Build a neutral model

> **Crates used:** `tpt-c-model`, `tpt-c-ids`, `tpt-c-units`, `tpt-c-core`, `serde_json`

Every engine in tpt-construction consumes the same neutral model
(`tpt-c-model`): parsers (IFC, BCF, LAS, DXF) produce it, and takeoff,
estimating, and export consume it. This recipe builds one by hand and
serializes it to the JSON understood by the `tpt` CLI.

## Cargo.toml

```toml
[dependencies]
tpt-c-core = "0.1"
tpt-c-ids = "0.1"
tpt-c-model = "0.1"
tpt-c-units = "0.1"
serde_json = "1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use tpt_c_core::ProjectId;
use tpt_c_ids::{ExternalId, IdFactory};
use tpt_c_model::{
    Element, Project, PropertySet, PropertyValue, Quantity, QuantitySet,
};
use tpt_c_units::{Area, Volume};

fn main() {
    // The project is the root aggregate of the neutral model.
    let mut project = Project::new(ProjectId::nil(), "Riverside Outbuilding");

    // Elements carry a stable id (deterministic here so reruns reproduce the
    // same JSON), a display name, and a category. The category doubles as the
    // default cost code when no classification is attached (see recipe 3).
    let slab = Element::new(
        IdFactory::deterministic_element("slab-1"),
        "SLAB-1",
        "Slab",
    )
    .with_quantity_set(
        // Explicit quantities flow straight through takeoff. The engine can
        // also derive volumes/areas from material layers — see recipe 2.
        QuantitySet::new("BaseQuantities").with(
            "GrossVolume",
            Quantity::Volume(Volume::from_cubic_meters(9.6)),
        ),
    )
    .with_property_set(
        PropertySet::new("Pset_SlabCommon")
            .with("FireRating", PropertyValue::Text("2HR".into())),
    );
    project.add_element(slab);

    // External ids map authoring-tool identifiers onto the internal one, so
    // round-trips through Revit/IFC keep their identity.
    let wall = Element::new(
        IdFactory::deterministic_element("wall-1"),
        "WALL-1",
        "Wall",
    )
    .with_external_id(ExternalId::RevitId("3355887".into()))
    .with_quantity_set(
        QuantitySet::new("BaseQuantities").with(
            "NetArea",
            Quantity::Area(Area::from_square_meters(48.0)),
        ),
    );
    project.add_element(wall);

    // Serialize to the neutral JSON the `tpt` CLI and format parsers speak.
    let json = serde_json::to_string_pretty(&project).unwrap();
    std::fs::write("model.json", &json).unwrap();
    println!(
        "wrote model.json: {} elements",
        project.element_count()
    );
}
```

## Run it

```bash
cargo run
```

```text
wrote model.json: 2 elements
```

## How it works

1. `Project::new` builds the root aggregate; ids come from `tpt-c-core`
   newtypes (`ProjectId`, `ElementId`) so ids of different entity types can
   never be mixed up.
2. `IdFactory::deterministic_element("slab-1")` derives a UUIDv5 from a name
   inside the TPT namespace — the same name always yields the same id, which
   makes fixtures and diffs reproducible. Swap in `IdFactory::element()` for
   time-ordered UUIDv7 ids in production.
3. Builder-style methods (`with_quantity_set`, `with_property_set`,
   `with_external_id`, `with_layer`, `classified`, `in_storey`) compose an
   element in one expression.
4. The serialized JSON is exactly the format in
   `test-data/golden/sample-model.json` — the `tpt estimate` subcommand and
   the `quantity-takeoff` example consume it directly.
