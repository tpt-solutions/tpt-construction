# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `MassHaulDiagram` with `new`, `add_station`, `total_cut`, `total_fill`, `net_cut`, `loose_volume`, and `compacted_volume`.
- `StationVolume` per-station cut/fill record with `new` and `net`.
- `SwellShrink` bank→loose / bank→compacted factors with `Default` (25% swell, 12% shrink).
- `CutFillBalance::compute` with `cut_bank`, `fill_bank`, `excess_cut`, `haul_factor`, `has_excess_cut`, and `has_deficit`.
