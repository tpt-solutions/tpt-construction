# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `SensorReading` telemetry record (sensor id, ISO 8601 timestamp, value,
  unit) with a `new` constructor.
- `SensorMapping` linking an `AssetId` to its sensors, with a chainable
  `add_sensor` builder.
- `TwinState` live-state snapshot with `new`, `push_reading`, `add_mapping`
  and `latest_for` (most recent reading per sensor).
- Serde `Serialize`/`Deserialize` support and `Default` on all public types.
