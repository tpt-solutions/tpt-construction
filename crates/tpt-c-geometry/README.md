# tpt-c-geometry

Geometry primitives for construction models: points, vectors, triangles,
axis-aligned bounding boxes, triangulated meshes and solids with area/volume
computation, a uniform-grid `SpatialIndex` for broad-phase queries, and
box-based clash detection. The crate is deliberately self-contained — no
external linear-algebra dependency — and is the geometry substrate shared by
the model, IFC, glTF, alignment, and earthwork crates.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Point3` (metres) and `Vec3` with dot/cross/length/normalize, `Add`/`Sub`
  operators, scalar multiplication, and `Point3 - Point3 -> Vec3`
- `Triangle` with `area`, `normal`, `unit_normal`, and `centroid`
- `BoundingBox` with `from_corners`, `empty`, `from_point`, `expand_point`,
  `union`, `intersects`, `contains_point`, `center`, `extents`, `volume`,
  and `surface_area`
- `Mesh` (shared vertices + indexed triangles, optional owner `ElementId`)
  with `iter_triangles`, `area`, `triangle_area`, closed-mesh `volume` via
  the divergence theorem, `bounds`, `translate`, and `surfaces`
- `Solid` composed of one or more closed shells with aggregate
  `volume`, `surface_area`, and `bounds`
- `SpatialIndex<T>` uniform-grid index for overlap queries
- `detect_clashes` producing `Clash<I>` records with overlap volume

## Usage

```toml
[dependencies]
tpt-c-geometry = "0.1"
```

```rust
use tpt_c_geometry::{detect_clashes, BoundingBox, Point3, SpatialIndex, Vec3};

let a = Vec3::new(1.0, 0.0, 0.0);
let b = Vec3::new(0.0, 2.0, 0.0);
assert_eq!(a.cross(b), Vec3::new(0.0, 0.0, 2.0));

let near = BoundingBox::from_corners(Point3::origin(), Point3::new(1.0, 1.0, 1.0));
let mut idx: SpatialIndex<usize> = SpatialIndex::new(1.0);
idx.insert(near, 0);
idx.insert(
    BoundingBox::from_corners(Point3::new(5.0, 5.0, 5.0), Point3::new(6.0, 6.0, 6.0)),
    1,
);
let hits = idx.query(&BoundingBox::from_corners(
    Point3::new(0.5, 0.5, 0.5),
    Point3::new(0.6, 0.6, 0.6),
));
assert_eq!(hits, vec![0]);

let boxes = vec![
    ("a", near),
    ("b", BoundingBox::from_corners(Point3::new(0.5, 0.5, 0.5), Point3::new(1.5, 1.5, 1.5))),
];
let clashes = detect_clashes(&boxes);
assert_eq!(clashes.len(), 1);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-core` (for `ElementId` mesh/solid
  ownership), `tpt-c-units` (for `Length`/`Area`/`Volume` measures)
- **Used by:** `tpt-c-alignment`, `tpt-c-earthwork`, and `tpt-c-wasm`
  (which re-exports `Point3`, `Vec3`, `Mesh`, and `BoundingBox` for the
  browser API)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
