# tpt-c-geo

Geospatial coordinate handling for construction sites. Provides coordinate
reference systems identified by EPSG code, geographic (lat/long/elevation)
coordinates, a project-local site grid (east/north/elevation), common datums,
survey grid-line definitions for staking, and the transform between a
project's local grid and WGS84 via a small-area tangent-plane approximation
suitable for site and corridor work.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Crs` with the `WGS84` (EPSG:4326) and `WEB_MERCATOR` (EPSG:3857)
  constants and `from_epsg` for custom systems
- `GeographicCoordinate` (decimal degrees + elevation in metres) and
  `LocalGridCoordinate` (east/north/elevation)
- `Datum` enum: `Wgs84`, `Nad83`, `Osgb36`, `Local`
- `SiteDatum` anchoring a local grid to a geographic origin with optional
  clockwise rotation; `to_local` and `to_geo` form a lossless round trip at
  site scale
- `GridLine` definitions (label, orientation, offset) for staking layout
- Pure, serializable, deterministic — only `serde` as a dependency

## Usage

```toml
[dependencies]
tpt-c-geo = "0.1"
```

```rust
use tpt_c_geo::{GeographicCoordinate, SiteDatum};

let datum = SiteDatum::new(GeographicCoordinate {
    latitude: 40.0,
    longitude: -75.0,
    elevation: 10.0,
});
let geo = GeographicCoordinate {
    latitude: 40.0009,
    longitude: -74.9987,
    elevation: 12.5,
};

let local = datum.to_local(geo);
assert!((local.elevation - 2.5).abs() < 1e-9);
let back = datum.to_geo(local);
assert!((back.latitude - geo.latitude).abs() < 1e-6);
assert!((back.longitude - geo.longitude).abs() < 1e-6);
```

## Crate relationships

- **Depends on:** `serde` only (no workspace dependencies)
- **Used by:** `tpt-c-alignment` and `tpt-c-earthwork`

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
