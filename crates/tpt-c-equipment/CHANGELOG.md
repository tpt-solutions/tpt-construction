# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `EquipmentRegistry` in-memory fleet store: `register` (duplicate asset tag
  rejected), `register_new` (mints `AssetId` via `IdFactory`), `get`, `get_mut`,
  `all`, `record_fuel`, `total_fuel_litres`, `record_utilization`,
  `average_utilization`, `set_maintenance_plan`, `ingest_telematics`, and
  `check_maintenance`.
- `Equipment` with builder methods `with_category` / `with_specs` / `with_status`,
  plus `EquipmentStatus::is_available`, `EquipmentCategory` presets
  (`earthmoving`, `lifting`, `hauling`), and `hours_since_service`.
- Telematics model: `TelematicsEvent`, `TelematicsReading` (`CanBusFrame`,
  `GpsFix`, `FuelLevel`, `EngineHours`, `IdleState`), `CanBusFrame` (J1939
  source address, PGN, SPN), and `GpsFix`.
- `FuelLogEntry` with `from_litres` / `litres` backed by `tpt_c_units::Volume`.
- `UtilizationRecord` with `total_hours` / `utilization_rate`.
- `MaintenancePlan` (`new`, `is_due`) and `MaintenanceTrigger` /
  `MaintenanceReason` (`HourInterval`, `DayInterval`).
- `EquipmentError` (`NotFound`, `DuplicateAssetTag`, `InvalidInterval`).
