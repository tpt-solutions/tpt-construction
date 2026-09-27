# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Asset` record with serial number, warranty expiry, install date and
  `replaced_by` successor link, plus `new`, `with_serial`, `with_warranty`,
  `with_install_date`, `replace` and `is_active`.
- `AssetStatus` lifecycle enum (`Active`, `Standby`, `UnderMaintenance`,
  `Retired`, `Replaced`) serialized as snake_case.
- Serde `Serialize`/`Deserialize` support on all public types.
