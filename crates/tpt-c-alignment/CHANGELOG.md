# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Point2D` plan-view point with `new` and `distance_to` (typed `Length`).
- `HorizontalElement` enum (`Tangent`, `Circular`, `Spiral`) with `length`.
- `VerticalElement` enum (`Grade`, `VerticalCurve`) with `elevation_at` and `length`.
- `Station` stationing newtype with `from_metres`, `metres`, and `format`.
- `Superelevation` cross-slope definition with `new` and `rate_at` linear runoff transition.
