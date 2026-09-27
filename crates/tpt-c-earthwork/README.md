# tpt-c-earthwork

Earthwork volumes, shrink/swell, mass haul, and volume balancing. The crate
provides the civil/earthwork primitives needed for cut/fill analysis and haul
planning: a `MassHaulDiagram` of per-station cut/fill records with totals and
net volume, `SwellShrink` factors converting bank volumes to loose or compacted
states, `StationVolume` records with signed net, and a `CutFillBalance` check
that tells you whether surplus cut must be hauled to spoil or borrow material
is required. Reach for it when corridor earthworks need volumetric accounting —
pair it with `tpt-c-alignment` to sample cut/fill along a design profile.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `MassHaulDiagram::add_station`: build a cumulative mass haul diagram from stationed cut/fill data
- Totals and balance: `total_cut`, `total_fill`, and `net_cut` (positive = leftover cut to haul away)
- `SwellShrink` factors (defaults: 25% swell, 12% shrink) with `loose_volume` and `compacted_volume` conversions via `tpt-c-units::Volume`
- `StationVolume::net`: signed cut-minus-fill per station
- `CutFillBalance::compute(cut, fill, haul_factor)` with `has_excess_cut` / `has_deficit` flags
- Pure, deterministic computation — serializable and WASM-friendly

## Usage

```toml
[dependencies]
tpt-c-earthwork = "0.1"
```

```rust
use tpt_c_earthwork::{CutFillBalance, MassHaulDiagram, SwellShrink};
use tpt_c_units::Volume;

let mut mhd = MassHaulDiagram::new(SwellShrink::default());
mhd.add_station(0.0, 100.0, 60.0);
mhd.add_station(50.0, 80.0, 90.0);
assert_eq!(mhd.total_cut(), 180.0);
assert_eq!(mhd.total_fill(), 150.0);
assert_eq!(mhd.net_cut(), 30.0);

let bank = Volume::from_cubic_meters(100.0);
assert!((mhd.loose_volume(bank).cubic_meters() - 125.0).abs() < 1e-9);
assert!((mhd.compacted_volume(bank).cubic_meters() - 88.0).abs() < 1e-9);

let b = CutFillBalance::compute(200.0, 180.0, 1.15);
assert!(b.has_excess_cut());
assert!(!b.has_deficit());
assert_eq!(b.excess_cut, 20.0);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-geo`, `tpt-c-geometry`, `tpt-c-units`
- **Used by:** the `earthwork-cut-fill` example (corridor stakeout, profile comparison, and mass haul reporting)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
