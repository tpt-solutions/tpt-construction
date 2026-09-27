# tpt-c-ifc

IFC (ISO 10303-21 / STEP) parser and mapper to the neutral `tpt-c-model`.
It parses the STEP physical file grammar into typed entities, then maps the
building elements needed for quantity takeoff and coordination —
`IfcProject`, `IfcSite`, `IfcBuilding`, `IfcBuildingStorey`, `IfcWall`,
`IfcSlab`, `IfcColumn`, `IfcBeam`, `IfcDoor`, `IfcWindow`, `IfcSpace` —
into a `tpt_c_model::Project`, carrying over property sets, element
quantities, and storey containment. Geometry (B-rep / tessellation) is not
decoded; the parser extracts the semantic structure only.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `parse`: a dependency-free ISO-10303-21 parser producing a `StepDoc`
  of `StepEntity` instances (`#id = TYPE(...)`)
- `StepValue` covering refs, strings, reals, ints, enumerations, lists,
  `$` (unspecified), and `*` (star); `StepEntity` accessors `str_at`,
  `real_at`, `int_at`, `ref_at`, and `list_at`
- `StepDoc` lookups by instance id (`get`) and case-insensitive by type
  (`by_type`, `by_type_prefix`)
- `to_model`: maps a `StepDoc` into a neutral `tpt_c_model::Project` with
  deterministic element ids derived from IFC GUIDs, `IfcGuid` external ids,
  storey assignment via `IfcRelContainedInSpatialStructure`, property sets
  via `IfcRelDefinesByProperties` / `IfcPropertySingleValue`, and quantity
  sets via `IfcElementQuantity` (`IfcQuantityLength/Area/Volume/Count/Weight`)
- `IfcError` for invalid headers, parse errors with offsets, dangling
  references, and a missing `IfcProject`

## Usage

```toml
[dependencies]
tpt-c-ifc = "0.1"
```

```rust
use tpt_c_ifc::{parse, to_model};

const SAMPLE: &str = r#"
ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('ViewDefinition [CoordinationView]'),'2;1');
FILE_NAME('sample.ifc','2026-01-01T00:00:00',('TPT'),('TPT'),'','','');
FILE_SCHEMA(('IFC2X3'));
ENDSEC;
DATA;
#1=IFCPROJECT('0YcC700kj2vxrGBVxoQqhl',$,'Demo Project',$,$,$,$,$,$);
#20=IFCWALL('wall-1',$,'East Wall',$,$,$,$,$,$,$);
ENDSEC;
END-ISO-10303-21;
"#;

let doc = parse(SAMPLE).expect("parse");
assert_eq!(doc.by_type("IFCWALL").len(), 1);
let project = to_model(&doc).expect("map");
assert_eq!(project.name, "Demo Project");
assert_eq!(project.element_count(), 1);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-classification`,
  `tpt-c-core`, `tpt-c-ids`, `tpt-c-model`, `tpt-c-units`
- **Used by:** `tpt-c-wasm` and the `ifc-import` and `wasm-browser-demo`
  examples

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
