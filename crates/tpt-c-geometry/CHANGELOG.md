# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Point3` (`new`, `origin`, `translate`, vector subtraction) and `Vec3`
  (`new`, `zero`, `distance`, `dot`, `cross`, `length_sq`, `length`,
  `normalize`, `Add`/`Sub`/`Mul<f64>`, `From<Point3>`)
- `Triangle` with `new`, `area`, `normal`, `unit_normal`, `centroid`
- `BoundingBox` with `from_corners`, `empty`, `from_point`, `expand_point`,
  `union`, `intersects`, `contains_point`, `center`, `extents`, `volume`,
  `surface_area`
- `Mesh` with `new`, `with_owner`, `triangle_count`, `iter_triangles`,
  `area`, `triangle_area`, `volume`, `bounds`, `translate`, `surfaces`
- `Solid` with `new`, `with_owner`, `bounds`, `volume`, `surface_area`
- `SpatialIndex<T>` uniform-grid broad-phase index with `new`, `insert`,
  `query`
- `Clash<I>` and `detect_clashes` pairwise box-overlap detection with
  overlap volumes
