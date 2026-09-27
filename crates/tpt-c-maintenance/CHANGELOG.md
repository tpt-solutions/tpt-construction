# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `WorkOrder` with `new`, `transition` (status changes) and `add_entry`
  (service history), targeting an `AssetId`.
- `MaintenanceKind` enum (`Preventive`, `Corrective`, `Inspection`,
  `Modification`) and `MaintenanceStatus` enum (`Requested`, `Scheduled`,
  `InProgress`, `Completed`, `Cancelled`), serialized as snake_case.
- `ServiceEntry` service-history record with `new` and `with_notes` builders.
- Serde `Serialize`/`Deserialize` support on all public types.
