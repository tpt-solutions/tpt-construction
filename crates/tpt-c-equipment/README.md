# tpt-c-equipment

Equipment registry, utilization, fuel and maintenance tracking, and field
telematics ingestion for construction plant and vehicles. The crate models a
fleet addressed by `AssetId`; telemetry arrives as `TelematicsEvent`s (J1939/CAN
bus frames, GPS fixes, fuel and engine-hour readings, idle state) and is folded
into the registry's live state, which drives utilization analytics and
maintenance triggers.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `EquipmentRegistry`: in-memory fleet store with duplicate-asset-tag protection,
  `register` / `register_new`, `get` / `get_mut`, and iteration via `all()`.
- `Equipment` builder-style construction (`with_category`, `with_specs`,
  `with_status`), `EquipmentStatus::is_available()`, and `hours_since_service()`.
- Telematics ingestion: `TelematicsReading` variants (`CanBusFrame` J1939 PGN/SPN
  frames, `GpsFix`, `FuelLevel`, `EngineHours`, `IdleState`) folded into live
  equipment state by `ingest_telematics`.
- Fuel logging with `FuelLogEntry::from_litres` (canonical `Volume` storage) and
  per-asset `total_fuel_litres`.
- `UtilizationRecord` (`total_hours` / `utilization_rate`) aggregated by
  `EquipmentRegistry::average_utilization`.
- Maintenance scheduling: `MaintenancePlan::new` (hour/day intervals), `is_due`,
  and `check_maintenance` producing `MaintenanceTrigger`s.
- Serializable with serde; a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-equipment = "0.1"
```

```rust
use tpt_c_equipment::{
    EquipmentRegistry, FuelLogEntry, MaintenancePlan, TelematicsEvent, TelematicsReading,
    UtilizationRecord,
};

let mut reg = EquipmentRegistry::new();
let id = reg.register_new("Excavator 1", "EXC-001").unwrap();

// Fuel and utilization.
reg.record_fuel(id, FuelLogEntry::from_litres("2026-01-01T00:00:00Z", 200.0))
    .unwrap();
let rec = UtilizationRecord {
    start: "2026-01-01T00:00:00Z".into(),
    end: "2026-01-02T00:00:00Z".into(),
    operating_hours: 8.0,
    idle_hours: 2.0,
};
reg.record_utilization(id, rec).unwrap();
assert!((reg.average_utilization(id) - 0.8).abs() < 1e-9);

// Telematics folds into live state.
reg.ingest_telematics(&TelematicsEvent {
    equipment_id: id,
    reading: TelematicsReading::EngineHours { hours: 1200.0 },
})
.unwrap();
assert_eq!(reg.get(id).unwrap().accumulated_hours, 1200.0);

// Maintenance trigger after 400 engine hours.
reg.set_maintenance_plan(id, MaintenancePlan::new(400.0, 0.0).unwrap())
    .unwrap();
let triggers = reg.check_maintenance(|_| 0.0);
assert!(!triggers.is_empty());
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core` (`AssetId`),
  `tpt-c-ids` (`IdFactory`), and `tpt-c-units` (`Volume`).
- **Used by:** `tpt-c-twin` declares it as a workspace dependency (no direct code
  references yet); see the `examples/` directory for integration patterns such as
  replicating equipment state with `tpt-c-sync`.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
