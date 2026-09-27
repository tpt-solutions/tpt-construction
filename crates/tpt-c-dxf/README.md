# tpt-c-dxf

Minimal ASCII DXF (Drawing eXchange Format) reader for 2D/3D CAD entities. Parses
the group-code/value text structure and extracts the entities that matter for
construction takeoff and clash work — LINE, LWPOLYLINE, CIRCLE, POINT, TEXT —
into plain coordinate arrays, with unknown entity types preserved by name. Any
other CAD payload (layers, blocks, headers) is intentionally skipped.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `DxfDocument::parse(&str)` — read the group-code/value pairs of an ASCII DXF and
  collect entities from the ENTITIES section
- Typed entities: `DxfEntity::Line` (3D), `LwPolyline` (2D vertices + closed flag),
  `Circle` (XY plane), `Point` (3D), `Text` (insertion point + string)
- Unknown entity types kept as `DxfEntity::Other { name }` instead of dropping them
- `segments()` — flatten everything to start/end point pairs: raw lines, polyline
  edges (including the closing edge), and 24-segment circle approximations
- `len()` / `is_empty()` for quick entity counts
- `DxfError`: `NoEntities`, `BadGroupCode`, `UnexpectedEof`
- Only dependency is `thiserror`; geometry stays in plain `[f64; 2]` / `[f64; 3]` arrays

## Usage

```toml
[dependencies]
tpt-c-dxf = "0.1"
```

```rust
use tpt_c_dxf::{DxfDocument, DxfEntity};

let src = std::fs::read_to_string("plan.dxf").unwrap();
let doc = DxfDocument::parse(&src).unwrap();
println!("{} entities", doc.len());

for entity in &doc.entities {
    if let DxfEntity::Line { start, end } = entity {
        println!("line {start:?} -> {end:?}");
    }
}

// Flatten to segments for length takeoff or clash checks:
// lines, polyline edges, and 24-gon circle approximations.
let segments = doc.segments();
```

## Crate relationships

- **Depends on:** `thiserror`
- **Used by:** Not yet consumed by other workspace crates; see the examples/
  directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) /
[LICENSE-APACHE](../../LICENSE-APACHE)).
