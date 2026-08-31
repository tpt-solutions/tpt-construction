// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Equipment registry, utilization, fuel and maintenance tracking, and field
//! telematics ingestion for construction plant and vehicles.
//!
//! The crate models a fleet of equipment addressed by the [`AssetId`] defined in
//! [`tpt_c_core`]. Telemetry arrives as [`TelematicsEvent`]s (J1939/CAN bus
//! frames, GPS fixes, fuel and engine-hour readings) and is folded into the
//! registry's live state, which drives utilization analytics and maintenance
//! triggers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tpt_c_core::AssetId;
use tpt_c_ids::IdFactory;
use tpt_c_units::Volume;

/// Errors produced while managing the equipment registry.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EquipmentError {
    /// No equipment with the given identifier is registered.
    #[error("equipment not found: {0}")]
    NotFound(AssetId),
    /// Two pieces of equipment share the same asset tag.
    #[error("duplicate asset tag: {0}")]
    DuplicateAssetTag(String),
    /// A maintenance interval was configured with a non-positive value.
    #[error("invalid maintenance interval: {0}")]
    InvalidInterval(&'static str),
}

/// Operating state of a piece of equipment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentStatus {
    /// Deployed and productive.
    Active,
    /// On site but not currently productive.
    Idle,
    /// Undergoing scheduled or unscheduled service.
    InMaintenance,
    /// Being relocated between sites.
    InTransit,
    /// Retired from the fleet.
    Retired,
}

impl EquipmentStatus {
    /// Whether the equipment can be assigned productive work.
    pub fn is_available(self) -> bool {
        matches!(self, EquipmentStatus::Active | EquipmentStatus::Idle)
    }
}

/// A coarse equipment class used for fleet reporting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentCategory(pub String);

impl EquipmentCategory {
    /// An earthmoving class (excavator, dozer, loader, ...).
    pub fn earthmoving() -> Self {
        Self("Earthmoving".into())
    }
    /// A lifting class (crane, hoist, ...).
    pub fn lifting() -> Self {
        Self("Lifting".into())
    }
    /// A hauling class (truck, dump, ...).
    pub fn hauling() -> Self {
        Self("Hauling".into())
    }
}

/// A registered piece of equipment in the fleet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    /// Stable asset identifier.
    pub id: AssetId,
    /// Human-readable name / number.
    pub name: String,
    /// Printed asset tag.
    pub asset_tag: String,
    /// Equipment class.
    pub category: EquipmentCategory,
    /// Manufacturer, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    /// Model designation, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Serial number, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    /// Current operating state.
    pub status: EquipmentStatus,
    /// Latest known GPS position, if telematics has reported one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<GpsFix>,
    /// Total accumulated engine hours.
    #[serde(default)]
    pub accumulated_hours: f64,
    /// Engine hours at the last completed service.
    #[serde(default)]
    pub last_service_hours: f64,
}

impl Equipment {
    /// Register a new piece of equipment.
    pub fn new(id: AssetId, name: impl Into<String>, asset_tag: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            asset_tag: asset_tag.into(),
            category: EquipmentCategory("General".into()),
            manufacturer: None,
            model: None,
            serial_number: None,
            status: EquipmentStatus::Idle,
            position: None,
            accumulated_hours: 0.0,
            last_service_hours: 0.0,
        }
    }

    /// Assign the equipment class.
    pub fn with_category(mut self, category: EquipmentCategory) -> Self {
        self.category = category;
        self
    }

    /// Attach manufacturer / model / serial metadata.
    pub fn with_specs(
        mut self,
        manufacturer: impl Into<String>,
        model: impl Into<String>,
        serial_number: impl Into<String>,
    ) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self.model = Some(model.into());
        self.serial_number = Some(serial_number.into());
        self
    }

    /// Set the current operating state.
    pub fn with_status(mut self, status: EquipmentStatus) -> Self {
        self.status = status;
        self
    }

    /// Hours elapsed since the last service.
    pub fn hours_since_service(&self) -> f64 {
        (self.accumulated_hours - self.last_service_hours).max(0.0)
    }
}

/// A single J1939 CAN-bus frame decoded from OEM telematics.
///
/// Heavy-equipment telemetry is typically exposed over SAE J1939. A frame is
/// identified by its Parameter Group Number (PGN) and a Suspect Parameter
/// Number (SPN) within that group, carrying a scalar value from a source
/// address on the bus.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanBusFrame {
    /// Source address on the J1939 network.
    pub source_address: u8,
    /// Parameter Group Number.
    pub pgn: u32,
    /// Suspect Parameter Number.
    pub spn: u32,
    /// Decoded scalar value.
    pub value: f64,
    /// RFC 3339 timestamp of the reading.
    pub at: String,
}

/// A GNSS/GPS position fix.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GpsFix {
    /// Latitude in decimal degrees (WGS84).
    pub latitude: f64,
    /// Longitude in decimal degrees (WGS84).
    pub longitude: f64,
    /// Elevation in metres, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elevation_m: Option<f64>,
    /// Ground speed in km/h, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed_kmh: Option<f64>,
    /// RFC 3339 timestamp of the fix.
    pub at: String,
}

/// A decoded telematics reading folded into equipment state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload", rename_all = "snake_case")]
pub enum TelematicsReading {
    /// A raw J1939 frame from the equipment bus.
    CanBusFrame(CanBusFrame),
    /// A GPS position fix.
    GpsFix(GpsFix),
    /// Reported fuel level in litres.
    FuelLevel {
        /// Fuel remaining in litres.
        litres: f64,
    },
    /// Cumulative engine hours reported by the controller.
    EngineHours {
        /// Total engine hours.
        hours: f64,
    },
    /// Idle / productive state as reported by the machine.
    IdleState {
        /// `true` when the machine reports idle.
        idle: bool,
    },
}

/// A timestamped telematics event for one piece of equipment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelematicsEvent {
    /// Equipment the reading belongs to.
    pub equipment_id: AssetId,
    /// The decoded reading.
    pub reading: TelematicsReading,
}

/// A fuel log entry (a fill or top-up event).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelLogEntry {
    /// RFC 3339 timestamp of the fill.
    pub at: String,
    /// Volume dispensed, stored canonically in cubic metres.
    pub volume: Volume,
    /// Cost per litre, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit_cost: Option<f64>,
    /// Engine hours at the time of the fill, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub odometer_hours: Option<f64>,
}

impl FuelLogEntry {
    /// Record a fill measured in litres.
    pub fn from_litres(at: impl Into<String>, litres: f64) -> Self {
        Self {
            at: at.into(),
            volume: Volume::from_cubic_meters(litres / 1000.0),
            unit_cost: None,
            odometer_hours: None,
        }
    }

    /// The dispensed volume expressed in litres.
    pub fn litres(&self) -> f64 {
        self.volume.cubic_meters() * 1000.0
    }
}

/// A utilization record summarising a period of operation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UtilizationRecord {
    /// RFC 3339 start of the period.
    pub start: String,
    /// RFC 3339 end of the period.
    pub end: String,
    /// Productive operating hours in the period.
    pub operating_hours: f64,
    /// Idle hours in the period.
    pub idle_hours: f64,
}

impl UtilizationRecord {
    /// Total hours (operating + idle) in the period.
    pub fn total_hours(&self) -> f64 {
        self.operating_hours + self.idle_hours
    }

    /// Utilization rate in `[0, 1]`: operating over total hours.
    ///
    /// Returns `0.0` when there are no recorded hours.
    pub fn utilization_rate(&self) -> f64 {
        let total = self.total_hours();
        if total <= 0.0 {
            0.0
        } else {
            self.operating_hours / total
        }
    }
}

/// Maintenance scheduling parameters for a piece of equipment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenancePlan {
    /// Service interval measured in engine hours (0 = disabled).
    pub interval_hours: f64,
    /// Service interval measured in calendar days (0 = disabled).
    pub interval_days: f64,
}

impl MaintenancePlan {
    /// Build a plan from hour and day intervals.
    pub fn new(interval_hours: f64, interval_days: f64) -> Result<Self, EquipmentError> {
        if interval_hours < 0.0 || interval_days < 0.0 {
            return Err(EquipmentError::InvalidInterval("intervals must be non-negative"));
        }
        Ok(Self {
            interval_hours,
            interval_days,
        })
    }

    /// Whether the equipment is due (or overdue) for service given elapsed hours
    /// and days since the last service.
    pub fn is_due(&self, hours_since_service: f64, days_since_service: f64) -> bool {
        let by_hours = self.interval_hours > 0.0 && hours_since_service >= self.interval_hours;
        let by_days = self.interval_days > 0.0 && days_since_service >= self.interval_days;
        by_hours || by_days
    }
}

/// A maintenance trigger raised for a piece of equipment.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceTrigger {
    /// Equipment the trigger applies to.
    pub equipment_id: AssetId,
    /// Reason the trigger fired.
    pub reason: MaintenanceReason,
    /// Hours over the hour-based interval, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hours_over: Option<f64>,
}

/// Why a maintenance trigger fired.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceReason {
    /// Hour-based interval reached.
    HourInterval,
    /// Calendar-day interval reached.
    DayInterval,
}

/// An in-memory fleet registry tracking equipment, fuel, utilization and
/// maintenance state, with telematics ingestion.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EquipmentRegistry {
    equipment: HashMap<AssetId, Equipment>,
    asset_tag_index: HashMap<String, AssetId>,
    fuel_logs: HashMap<AssetId, Vec<FuelLogEntry>>,
    utilization: HashMap<AssetId, Vec<UtilizationRecord>>,
    maintenance_plans: HashMap<AssetId, MaintenancePlan>,
}

impl EquipmentRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a piece of equipment, returning its id.
    ///
    /// Fails if the asset tag is already in use by another asset.
    pub fn register(&mut self, equipment: Equipment) -> Result<AssetId, EquipmentError> {
        if self.asset_tag_index.contains_key(&equipment.asset_tag) {
            return Err(EquipmentError::DuplicateAssetTag(equipment.asset_tag.clone()));
        }
        let id = equipment.id;
        self.asset_tag_index.insert(equipment.asset_tag.clone(), id);
        self.equipment.insert(id, equipment);
        Ok(id)
    }

    /// Convenience constructor that mints a fresh [`AssetId`].
    pub fn register_new(
        &mut self,
        name: impl Into<String>,
        asset_tag: impl Into<String>,
    ) -> Result<AssetId, EquipmentError> {
        let id = IdFactory::asset();
        let eq = Equipment::new(id, name, asset_tag);
        self.register(eq)
    }

    /// Fetch a reference to registered equipment.
    pub fn get(&self, id: AssetId) -> Option<&Equipment> {
        self.equipment.get(&id)
    }

    /// Fetch a mutable reference to registered equipment.
    pub fn get_mut(&mut self, id: AssetId) -> Option<&mut Equipment> {
        self.equipment.get_mut(&id)
    }

    /// All registered equipment.
    pub fn all(&self) -> impl Iterator<Item = &Equipment> {
        self.equipment.values()
    }

    /// Attach a maintenance plan to a piece of equipment.
    pub fn set_maintenance_plan(
        &mut self,
        id: AssetId,
        plan: MaintenancePlan,
    ) -> Result<(), EquipmentError> {
        self.ensure(id)?;
        self.maintenance_plans.insert(id, plan);
        Ok(())
    }

    /// Record a fuel fill for the equipment.
    pub fn record_fuel(&mut self, id: AssetId, entry: FuelLogEntry) -> Result<(), EquipmentError> {
        self.ensure(id)?;
        self.fuel_logs.entry(id).or_default().push(entry);
        Ok(())
    }

    /// Total fuel dispensed (litres) for the equipment.
    pub fn total_fuel_litres(&self, id: AssetId) -> f64 {
        self.fuel_logs
            .get(&id)
            .map(|log| log.iter().map(FuelLogEntry::litres).sum())
            .unwrap_or(0.0)
    }

    /// Record a utilization period for the equipment.
    pub fn record_utilization(
        &mut self,
        id: AssetId,
        record: UtilizationRecord,
    ) -> Result<(), EquipmentError> {
        self.ensure(id)?;
        self.utilization.entry(id).or_default().push(record);
        Ok(())
    }

    /// Average utilization rate across all recorded periods.
    pub fn average_utilization(&self, id: AssetId) -> f64 {
        let records = match self.utilization.get(&id) {
            Some(r) if !r.is_empty() => r,
            _ => return 0.0,
        };
        let total: f64 = records.iter().map(UtilizationRecord::utilization_rate).sum();
        total / records.len() as f64
    }

    /// Ingest a telematics event, folding it into live equipment state.
    ///
    /// Engine-hour and GPS readings update the corresponding equipment fields;
    /// idle readings are reflected in the equipment status.
    pub fn ingest_telematics(&mut self, event: &TelematicsEvent) -> Result<(), EquipmentError> {
        let eq = self.get_mut(event.equipment_id).ok_or(EquipmentError::NotFound(event.equipment_id))?;
        match &event.reading {
            TelematicsReading::GpsFix(fix) => {
                eq.position = Some(fix.clone());
            }
            TelematicsReading::EngineHours { hours } => {
                eq.accumulated_hours = eq.accumulated_hours.max(*hours);
            }
            TelematicsReading::IdleState { idle } => {
                if eq.status == EquipmentStatus::Active || eq.status == EquipmentStatus::Idle {
                    eq.status = if *idle {
                        EquipmentStatus::Idle
                    } else {
                        EquipmentStatus::Active
                    };
                }
            }
            TelematicsReading::CanBusFrame(_) | TelematicsReading::FuelLevel { .. } => {}
        }
        Ok(())
    }

    /// Raise maintenance triggers for all equipment whose plans are due.
    ///
    /// `days_since_service` is taken from the caller (it depends on calendar
    /// state the registry does not itself track).
    pub fn check_maintenance(
        &self,
        days_since_service: impl Fn(AssetId) -> f64,
    ) -> Vec<MaintenanceTrigger> {
        let mut triggers = Vec::new();
        for (id, plan) in &self.maintenance_plans {
            if let Some(eq) = self.equipment.get(id) {
                let hours = eq.hours_since_service();
                let days = days_since_service(*id);
                if plan.is_due(hours, days) {
                    let (reason, over) = if plan.interval_hours > 0.0 && hours >= plan.interval_hours
                    {
                        (MaintenanceReason::HourInterval, Some(hours - plan.interval_hours))
                    } else {
                        (MaintenanceReason::DayInterval, None)
                    };
                    triggers.push(MaintenanceTrigger {
                        equipment_id: *id,
                        reason,
                        hours_over: over,
                    });
                }
            }
        }
        triggers
    }

    fn ensure(&self, id: AssetId) -> Result<(), EquipmentError> {
        if self.equipment.contains_key(&id) {
            Ok(())
        } else {
            Err(EquipmentError::NotFound(id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut reg = EquipmentRegistry::new();
        let id = reg.register_new("Excavator 1", "EXC-001").unwrap();
        assert!(reg.get(id).is_some());
        assert_eq!(reg.all().count(), 1);
    }

    #[test]
    fn duplicate_asset_tag_rejected() {
        let mut reg = EquipmentRegistry::new();
        reg.register_new("A", "TAG-1").unwrap();
        let err = reg.register_new("B", "TAG-1").unwrap_err();
        assert!(matches!(err, EquipmentError::DuplicateAssetTag(_)));
    }

    #[test]
    fn fuel_and_utilization() {
        let mut reg = EquipmentRegistry::new();
        let id = reg.register_new("Loader", "LD-1").unwrap();
        reg.record_fuel(id, FuelLogEntry::from_litres("2026-01-01T00:00:00Z", 200.0)).unwrap();
        reg.record_fuel(id, FuelLogEntry::from_litres("2026-01-02T00:00:00Z", 150.0)).unwrap();
        assert!((reg.total_fuel_litres(id) - 350.0).abs() < 1e-9);

        let rec = UtilizationRecord {
            start: "2026-01-01T00:00:00Z".into(),
            end: "2026-01-02T00:00:00Z".into(),
            operating_hours: 8.0,
            idle_hours: 2.0,
        };
        reg.record_utilization(id, rec).unwrap();
        assert!((reg.average_utilization(id) - 0.8).abs() < 1e-9);
    }

    #[test]
    fn telematics_updates_state() {
        let mut reg = EquipmentRegistry::new();
        let id = reg.register_new("Crane", "CR-1").unwrap();
        reg.ingest_telematics(&TelematicsEvent {
            equipment_id: id,
            reading: TelematicsReading::EngineHours { hours: 1200.0 },
        })
        .unwrap();
        reg.ingest_telematics(&TelematicsEvent {
            equipment_id: id,
            reading: TelematicsReading::GpsFix(GpsFix {
                latitude: 40.0,
                longitude: -75.0,
                elevation_m: Some(10.0),
                speed_kmh: Some(0.0),
                at: "2026-01-01T00:00:00Z".into(),
            }),
        })
        .unwrap();
        let eq = reg.get(id).unwrap();
        assert_eq!(eq.accumulated_hours, 1200.0);
        assert!(eq.position.is_some());
    }

    #[test]
    fn maintenance_trigger_fires() {
        let mut reg = EquipmentRegistry::new();
        let id = reg.register_new("Dozer", "DZ-1").unwrap();
        reg.get_mut(id).unwrap().accumulated_hours = 500.0;
        reg.set_maintenance_plan(id, MaintenancePlan::new(400.0, 0.0).unwrap()).unwrap();
        let triggers = reg.check_maintenance(|_| 0.0);
        assert_eq!(triggers.len(), 1);
        assert_eq!(triggers[0].reason, MaintenanceReason::HourInterval);
    }
}
