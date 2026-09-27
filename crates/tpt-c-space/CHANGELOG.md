# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Space` record with category, net/gross areas, floor and occupancy, plus
  `new`, `with_gross_area`, `with_floor` and `with_occupancy` builders.
- `Lease` record with tenant, ISO 8601 start/end dates and lettable area.
- `SpaceInventory` project-scoped collection with `new`, `add_space`,
  `add_lease`, `total_net_area` and `total_leased_area`.
- Serde `Serialize`/`Deserialize` support and `Default` on all public types.
