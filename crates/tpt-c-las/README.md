# tpt-c-las

ASPRS LAS point-cloud reading for survey and earthwork workflows. Provides a
dependency-free parser for the LAS public header block and point records (binary
point formats 0–3), exposing points in real-world coordinates (scale and offset
applied), plus ASPRS classification codes and a coarse bounding-box volume
estimate for stockpile/earthwork screening.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `read_las(&[u8]) -> (LasHeader, Vec<LasPoint>)` for LAS 1.x binary files, point formats 0–3
- `LasHeader`: version, point format and record length, point count, scale/offset, min/max bounds
- `LasPoint`: x/y/z in real-world units, intensity, return number, number of returns, classification
- `classification` module with ASPRS codes (`GROUND`, `BUILDING`, vegetation, water, ...)
- `approximate_volume(&LasHeader)`: deterministic bounding-box volume estimate
- `LasError`: `BadSignature`, `UnsupportedFormat`, `Truncated`, `InvalidNumber`
- Only dependency is `thiserror` — fully offline parsing, no external point-cloud libraries

## Usage

```toml
[dependencies]
tpt-c-las = "0.1"
```

```rust
use tpt_c_las::{approximate_volume, classification, read_las};

let bytes = std::fs::read("survey.las").unwrap();
let (header, points) = read_las(&bytes).unwrap();

assert_eq!(header.point_count as usize, points.len());
println!("bounds: {:?} .. {:?}", header.min, header.max);

// Ground points for a terrain surface; buildings for site context.
let ground: Vec<_> = points
    .iter()
    .filter(|p| p.classification == classification::GROUND)
    .collect();

println!("bounding-box volume: {}", approximate_volume(&header));
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
