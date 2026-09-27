# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Length` (m/ft/in/yd), `Area` (m²/SF/SY), `Volume` (m³/CY/cf), `Count`
  (each), `Duration` (h/d/wk), and `Mass` (kg/short ton/metric tonne)
  measures with `from_*` constructors, unit accessors, and `Add`/`Sub`
- `WasteFactor` (`none`, `from_percent`, `ratio`, `apply`) and the
  `Wasteable` trait implemented by `Length`, `Area`, `Volume`, `Count`,
  and `Mass`
- `SoilState` (`Bank`, `Loose`, `Compacted`) and `SoilVolume` with
  `new`, `cubic_yards`, and `convert` for swell/shrink conversion
- `rebar_unit_weight_kg_per_m` and `rebar_weight` for US bar sizes #3–#11
- `PaintCoverage` with `new`, `covered_area`, and `litres_for`
- `round_to_decimals` rounding half away from zero
