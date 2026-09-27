# 10. Digital twin telemetry

> **Crates used:** `tpt-c-equipment`, `tpt-c-twin`, `tpt-c-ids`

Register a small fleet, fold a live telematics feed (engine hours, GPS fixes, idle reports) into the equipment registry, mirror the same feed into the digital twin's sensor layer so dashboards can query latest readings per sensor, and finish with fuel, utilization and maintenance analytics. The registry is the operational system of record; the twin is the queryable live-state view on the same asset ids.

## Cargo.toml

```toml
[dependencies]
tpt-c-equipment = "0.1"
tpt-c-twin = "0.1"
tpt-c-ids = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

// Digital twin telemetry: a small fleet is registered, live telematics events
// (engine hours, GPS fixes, idle state) are folded into the equipment registry,
// the same feed is mirrored into a digital-twin sensor layer for queries, and
// fuel/utilization/maintenance analytics run on top.

use tpt_c_equipment::{
    Equipment, EquipmentCategory, EquipmentRegistry, EquipmentStatus, FuelLogEntry, GpsFix,
    MaintenancePlan, MaintenanceReason, TelematicsEvent, TelematicsReading, UtilizationRecord,
};
use tpt_c_ids::{AssetId, IdFactory};
use tpt_c_twin::{SensorMapping, SensorReading, TwinState};

fn main() {
    // ------------------------------------------------------------------
    // 1. Register the fleet. Deterministic ids (UUIDv5) keep the demo
    //    output stable and let the twin layer refer to the same assets.
    // ------------------------------------------------------------------
    let exc_id = AssetId::from_uuid(IdFactory::deterministic("exc-001"));
    let dz_id = AssetId::from_uuid(IdFactory::deterministic("dz-001"));

    let mut fleet = EquipmentRegistry::new();
    fleet
        .register(
            Equipment::new(exc_id, "Excavator EX-1", "EXC-001")
                .with_category(EquipmentCategory::earthmoving())
                .with_specs("Acme Heavy", "EX200", "SN-EX200-001")
                .with_status(EquipmentStatus::Active),
        )
        .expect("asset tag EXC-001 is unique");
    fleet
        .register(Equipment::new(dz_id, "Dozer DZ-1", "DZ-001"))
        .expect("asset tag DZ-001 is unique");
    println!("Fleet ready: 2 assets (excavator id {exc_id})");

    // ------------------------------------------------------------------
    // 2. Ingest the telematics feed. Each event folds into live state:
    //    engine hours accumulate, GPS fixes update position, idle
    //    reports drive the operating status.
    // ------------------------------------------------------------------
    let feed = [
        TelematicsEvent { equipment_id: exc_id, reading: TelematicsReading::EngineHours { hours: 505.0 } },
        TelematicsEvent { equipment_id: exc_id, reading: TelematicsReading::EngineHours { hours: 512.5 } },
        TelematicsEvent {
            equipment_id: exc_id,
            reading: TelematicsReading::GpsFix(GpsFix {
                latitude: 40.7128,
                longitude: -74.0060,
                elevation_m: Some(12.0),
                speed_kmh: Some(0.0),
                at: "2026-03-01T18:00:00Z".into(),
            }),
        },
        TelematicsEvent { equipment_id: exc_id, reading: TelematicsReading::IdleState { idle: false } },
    ];
    for event in &feed {
        fleet.ingest_telematics(event).expect("equipment is registered");
    }

    let exc = fleet.get(exc_id).expect("registered");
    println!(
        "Telematics: {} has {:.1} h on the clock, status {:?}",
        exc.asset_tag, exc.accumulated_hours, exc.status
    );
    let fix = exc.position.as_ref().expect("GPS fix ingested");
    println!(
        "Last seen: ({:.4}, {:.4}) at {}",
        fix.latitude, fix.longitude, fix.at
    );

    // ------------------------------------------------------------------
    // 3. Mirror the feed into the digital twin: readings keyed by sensor
    //    id, plus a mapping from each asset to its sensors so dashboards
    //    can ask "what is this asset reporting right now?"
    // ------------------------------------------------------------------
    let mut twin = TwinState::new();
    twin.push_reading(SensorReading::new("exc-001/engine-hours", "2026-03-01T17:00:00Z", 505.0, "h"));
    twin.push_reading(SensorReading::new("exc-001/engine-hours", "2026-03-01T18:00:00Z", 512.5, "h"));
    twin.push_reading(SensorReading::new("exc-001/fuel-level", "2026-03-01T18:00:00Z", 210.0, "L"));
    twin.add_mapping(
        SensorMapping::new(exc_id)
            .add_sensor("exc-001/engine-hours")
            .add_sensor("exc-001/fuel-level"),
    );
    twin.add_mapping(SensorMapping::new(dz_id).add_sensor("dz-001/engine-hours"));

    let latest = twin.latest_for("exc-001/engine-hours").expect("sensor has readings");
    println!(
        "Twin: latest engine hours = {:.1} {} (reported at {})",
        latest.value, latest.unit, latest.timestamp
    );

    // ------------------------------------------------------------------
    // 4. Fuel and utilization analytics for the shift.
    // ------------------------------------------------------------------
    fleet
        .record_fuel(exc_id, FuelLogEntry::from_litres("2026-03-01T06:00:00Z", 200.0))
        .expect("registered");
    fleet
        .record_fuel(exc_id, FuelLogEntry::from_litres("2026-03-01T12:30:00Z", 150.0))
        .expect("registered");
    fleet
        .record_utilization(
            exc_id,
            UtilizationRecord {
                start: "2026-03-01T06:00:00Z".into(),
                end: "2026-03-01T18:00:00Z".into(),
                operating_hours: 8.0,
                idle_hours: 2.0,
            },
        )
        .expect("registered");
    println!(
        "Analytics: {:.1} L fuel logged, average utilization {:.1}%",
        fleet.total_fuel_litres(exc_id),
        fleet.average_utilization(exc_id) * 100.0
    );

    // ------------------------------------------------------------------
    // 5. Maintenance: attach a service plan per asset, then evaluate
    //    them all. Calendar time is caller-supplied because the registry
    //    deliberately does not track wall-clock state.
    // ------------------------------------------------------------------
    fleet
        .set_maintenance_plan(exc_id, MaintenancePlan::new(400.0, 0.0).expect("valid plan"))
        .expect("registered");
    fleet
        .set_maintenance_plan(dz_id, MaintenancePlan::new(0.0, 30.0).expect("valid plan"))
        .expect("registered");

    // The dozer was serviced 45 days ago; the excavator 0 days ago.
    let days_since_service = |id: AssetId| if id == dz_id { 45.0 } else { 0.0 };
    let triggers = fleet.check_maintenance(days_since_service);

    // Iterate a fixed order: the registry stores plans in a HashMap.
    for id in [exc_id, dz_id] {
        let tag = fleet.get(id).expect("registered").asset_tag.as_str();
        match triggers.iter().find(|t| t.equipment_id == id) {
            Some(t) => match t.reason {
                MaintenanceReason::HourInterval => println!(
                    "Maintenance: {tag} due by hour interval ({:.1} h over)",
                    t.hours_over.unwrap_or(0.0)
                ),
                MaintenanceReason::DayInterval => {
                    println!("Maintenance: {tag} due by day interval")
                }
            },
            None => println!("Maintenance: {tag} OK"),
        }
    }
}
```

## Run it

```bash
cargo run
```

Expected output (the excavator id is a deterministic UUIDv5 of `"exc-001"`, and every other printed value comes from fixed inputs, so the run is stable):

```text
Fleet ready: 2 assets (excavator id a8e367bf-36b7-5932-909e-db2751670ffd)
Telematics: EXC-001 has 512.5 h on the clock, status Active
Last seen: (40.7128, -74.0060) at 2026-03-01T18:00:00Z
Twin: latest engine hours = 512.5 h (reported at 2026-03-01T18:00:00Z)
Analytics: 350.0 L fuel logged, average utilization 80.0%
Maintenance: EXC-001 due by hour interval (112.5 h over)
Maintenance: DZ-001 due by day interval
```

## How it works

1. **One asset id, two layers.** `Equipment::new(id, name, asset_tag)` plus the `with_category` / `with_specs` / `with_status` builders describe the machine; `EquipmentRegistry::register` indexes it by id and enforces unique asset tags. Both the registry and the twin address the machine by the same `AssetId` (here a deterministic UUIDv5 via `IdFactory::deterministic`), so a dashboard can join twin sensor data straight onto fleet records.
2. **Telematics is a fold, not a table write.** `ingest_telematics` matches on the `TelematicsReading` variant: `EngineHours` ratchets `accumulated_hours` forward with `max` (out-of-order frames cannot lose hours), `GpsFix` replaces the latest position, and `IdleState` flips the machine between `Idle` and `Active` — but only from those states, so a machine flagged `InMaintenance` is never overridden by a stale bus frame.
3. **The twin is a sensor-level view.** `TwinState` stores raw `SensorReading`s (sensor id, timestamp, value, unit) plus `SensorMapping`s from asset to sensor ids. `latest_for` scans backwards for the newest reading of a sensor, which is the query a live dashboard actually makes. Keeping readings decoupled from the registry means twin data can stream in faster than fleet state needs to change.
4. **Analytics come from recorded periods, not estimates.** `FuelLogEntry::from_litres` stores volumes canonically in cubic metres (`litres()` converts back), so `total_fuel_litres` is unit-safe across sources. `UtilizationRecord` splits the shift into operating vs idle hours; `average_utilization` averages the per-period rates — here 8.0 / (8.0 + 2.0) = 80%.
5. **Maintenance is policy + data.** `MaintenancePlan::new(hours, days)` validates non-negative intervals, and `check_maintenance` walks every plan, taking elapsed calendar days as a closure (the registry tracks engine hours itself but not wall-clock time). The excavator trips its 400-hour interval at 512.5 hours since service (112.5 h over, reason `HourInterval`); the plan-less hour interval on the dozer leaves its 30-day interval to fire (`DayInterval`). Iterating the fixed `[exc_id, dz_id]` order — rather than the triggers vector, whose order follows the registry's internal `HashMap` — keeps the output deterministic.
