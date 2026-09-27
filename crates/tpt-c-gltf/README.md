# tpt-c-gltf

Export neutral construction models ([`tpt-c-model`](../tpt-c-model)) to glTF 2.0
JSON for web and mesh viewers. The neutral `Project` carries no triangle soup, so
this exporter builds a glTF scene where each element becomes a node referencing a
shared unit-cube mesh, scaled to the element's dimensions (read from its property
sets when present) and annotated with its metadata in `extras`. Output is
standards-compliant glTF 2.0 JSON suitable for `<model-viewer>`, three.js, and
other web viewers.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `export(&Project) -> GltfDocument`: one glTF node per element plus a project root node
- Per-element scale from `Length` / `Height` / `Width` property values (unit-cube fallback)
- Element metadata mirrored into node `extras`: category, storey, external id, classification
- `GltfDocument::to_json()`: standards-compliant glTF 2.0 JSON (`asset`, `scene`/`scenes`,
  `nodes`, `meshes`, `accessors`, `bufferViews`, `buffers`)
- Single embedded geometry buffer as a base64 data URI: 8-position unit cube with
  12 triangles (36 `u16` indices), correct min/max accessors and ARRAY_BUFFER targets
- Hand-rolled base64 encoder — no external base64 or mesh-processing dependency

## Usage

```toml
[dependencies]
tpt-c-gltf = "0.1"
```

```rust
use tpt_c_core::ProjectId;
use tpt_c_gltf::export;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, PropertySet, PropertyValue};

let mut project = Project::new(ProjectId::nil(), "Demo");
project.add_element(
    Element::new(IdFactory::element(), "W1", "Wall").with_property_set(
        PropertySet::new("Dims")
            .with("Length", PropertyValue::Number(10.0))
            .with("Height", PropertyValue::Number(3.0))
            .with("Width", PropertyValue::Number(0.3)),
    ),
);

// Each element becomes a node scaling the shared unit-cube mesh.
let doc = export(&project);
let json = doc.to_json();
// `json` is valid glTF 2.0: asset.version == "2.0", two nodes
// (the wall + the project root), wall scale == [10.0, 3.0, 0.3].
```

## Crate relationships

- **Depends on:** `serde`, `serde_json`, `tpt-c-core`, `tpt-c-ids`, `tpt-c-model`,
  `tpt-c-units`
- **Used by:** [`tpt-c-wasm`](../tpt-c-wasm) (re-exports `export` in its public API)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
