# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `FacilityAsset` handover asset-register entry with `new`,
  `with_manufacturer` and `with_serial` builders over `AssetId`, name,
  system, space, manufacturer, model and serial number.
- `CobieComponent` COBie-style component record with `new` constructor.
- Serde `Serialize`/`Deserialize` support on all public types, with optional
  fields skipped when absent.
