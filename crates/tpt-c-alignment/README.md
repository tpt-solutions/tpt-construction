# tpt-c-alignment

Horizontal and vertical road/rail alignments, curves, superelevation,
stationing, and corridor modeling. The crate models plan-view alignments as
`HorizontalElement` chains (tangents, circular curves, clothoid-approximated
spirals) and profiles as `VerticalElement` chains (constant grades and
parabolic vertical curves with `elevation_at` sampling), with `Station`
formatting (`10+500.0`) and linear `Superelevation` runoff transitions. It
defines its own small plan-view `Point2D` (with `tpt-c-units::Length`
distances) rather than reusing the 3D primitives in `tpt-c-geometry` —
alignment work is 2D by nature. Reach for it to stake out or sample a corridor
design, e.g. before running cut/fill with `tpt-c-earthwork`.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `HorizontalElement`: `Tangent`, `Circular` (radius + sweep), and `Spiral` variants, each with a `length()`
- `VerticalElement`: `Grade` and parabolic `VerticalCurve` with `elevation_at(d)` sampling and `length()`
- `Point2D` with `distance_to` returning a typed `Length`
- `Station` (ordered metres from datum) with `from_metres`, `metres`, and `format()` rendering `km+00.0`
- `Superelevation` with `rate_at(d)` linear transition from normal crown to maximum rate
- Plain-data, serializable geometry — no IO, WASM-friendly

## Usage

```toml
[dependencies]
tpt-c-alignment = "0.1"
```

```rust
use tpt_c_alignment::{HorizontalElement, Point2D, Station, Superelevation, VerticalElement};

let tangent = HorizontalElement::Tangent {
    start: Point2D::new(0.0, 0.0),
    end: Point2D::new(300.0, 0.0),
};
let curve = HorizontalElement::Circular {
    start: Point2D::new(300.0, 0.0),
    center: Point2D::new(300.0, 200.0),
    radius: 200.0,
    sweep: std::f64::consts::FRAC_PI_2,
};
assert_eq!(tangent.length().meters(), 300.0);

let grade = VerticalElement::Grade {
    start_elevation: 100.0,
    grade: 0.03,
    length: 100.0,
};
assert_eq!(grade.elevation_at(100.0), 103.0);

assert_eq!(Station::from_metres(10500).format(), "10+500.0");

let se = Superelevation::new(0.06, 100.0, -0.02);
assert!((se.rate_at(50.0) - 0.02).abs() < 1e-9);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-geo`, `tpt-c-geometry`, `tpt-c-units`
- **Used by:** the `earthwork-cut-fill` example (corridor stakeout feeding the mass haul diagram)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
