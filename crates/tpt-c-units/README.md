# tpt-c-units

Construction measurement units and domain measure helpers. Internal storage
always uses SI base units (metres, square metres, cubic metres, hours,
kilograms, count) so arithmetic is unambiguous; named construction units —
linear feet (LF), square feet (SF), square yards (SY), cubic yards (CY),
each, hour/day/week, kilogram, short ton, and their metric counterparts —
are converted at the boundary. Reach for it whenever a workspace crate needs
a typed quantity, a waste-factor expansion, a soil swell/shrink conversion,
rebar weight, or paint coverage.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Length`, `Area`, `Volume`, `Count`, `Duration`, and `Mass` newtypes stored
  in SI base units, each with `from_*` constructors and unit accessors
  (`feet`, `inches`, `square_yards`, `cubic_yards`, `tons`, ...) plus `Add`
  and `Sub` operators
- `WasteFactor` (ratio or percent) and the `Wasteable` trait for expanding
  quantities by `(1 + waste)`
- `SoilVolume` with `SoilState` (`Bank` / `Loose` / `Compacted`) and
  `convert` for swell/shrink earthwork conversions
- `rebar_unit_weight_kg_per_m` and `rebar_weight` for US bar sizes #3–#11
- `PaintCoverage` for litres↔area coverage math
- `round_to_decimals` (half away from zero) for presentable takeoff numbers
- Only `serde` as a dependency; fully deterministic and IO-free

## Usage

```toml
[dependencies]
tpt-c-units = "0.1"
```

```rust
use tpt_c_units::{Area, Length, SoilState, SoilVolume, Volume, WasteFactor};

// Imperial to SI round-trip.
let l = Length::from_feet(10.0);
assert!((l.meters() - 3.048).abs() < 1e-9);
assert!((l.feet() - 10.0).abs() < 1e-9);

// Apply a 10% waste factor to 100 SF.
let w = WasteFactor::from_percent(10.0);
let q = w.apply(Area::from_square_feet(100.0));
assert!((q.square_feet() - 110.0).abs() < 1e-9);

// Soil swell/shrink: 100 CY bank -> 125 CY loose or 88 CY compacted.
let bank = SoilVolume::new(Volume::from_cubic_yards(100.0), SoilState::Bank);
let loose = bank.convert(SoilState::Loose, 0.25, 0.12);
assert!((loose.cubic_yards() - 125.0).abs() < 1e-6);
```

## Crate relationships

- **Depends on:** `serde` only (no workspace dependencies)
- **Used by:** `tpt-c-model`, `tpt-c-geometry`, `tpt-c-ifc`, `tpt-c-gltf`,
  `tpt-c-alignment`, `tpt-c-earthwork`, `tpt-c-cost`, `tpt-c-estimating`,
  `tpt-c-quantities`, `tpt-c-risk`, `tpt-c-schedule`, `tpt-c-equipment`, and
  `tpt-c-xlsx`, plus the `cpm-schedule` and `earthwork-cut-fill` examples and
  the `tpt` CLI (`serve` feature)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
