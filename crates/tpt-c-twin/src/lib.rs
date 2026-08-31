// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Digital twin state, sensor mapping, live telemetry, spatial asset context.
//!
//! Provides the live state layer for digital twins: a snapshot of asset state,
//! sensor readings mapped to assets, and telemetry event streams.

use serde::{Deserialize, Serialize};
use tpt_c_ids::AssetId;

/// A reading from a sensor at a point in time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensorReading {
    /// Sensor identifier.
    pub sensor_id: String,
    /// ISO 8601 timestamp of the reading.
    pub timestamp: String,
    /// Numeric value.
    pub value: f64,
    /// Unit of measure (e.g. "C", "kPa", "RPM").
    pub unit: String,
}

impl SensorReading {
    /// Build a sensor reading.
    pub fn new(
        sensor_id: impl Into<String>,
        timestamp: impl Into<String>,
        value: f64,
        unit: impl Into<String>,
    ) -> Self {
        Self {
            sensor_id: sensor_id.into(),
            timestamp: timestamp.into(),
            value,
            unit: unit.into(),
        }
    }
}

/// A mapping between an asset and its sensors.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SensorMapping {
    /// Asset being monitored.
    pub asset_id: AssetId,
    /// Sensor identifiers attached to this asset.
    pub sensor_ids: Vec<String>,
}

impl SensorMapping {
    /// Build a sensor mapping.
    pub fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            sensor_ids: Vec::new(),
        }
    }

    /// Attach a sensor id.
    pub fn add_sensor(mut self, sensor_id: impl Into<String>) -> Self {
        self.sensor_ids.push(sensor_id.into());
        self
    }
}

/// The live digital twin state for a set of assets.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TwinState {
    /// Latest sensor readings keyed by sensor id.
    pub readings: Vec<SensorReading>,
    /// Asset-to-sensor mappings.
    pub mappings: Vec<SensorMapping>,
}

impl TwinState {
    /// Build an empty twin state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new sensor reading.
    pub fn push_reading(&mut self, reading: SensorReading) {
        self.readings.push(reading);
    }

    /// Add a sensor mapping.
    pub fn add_mapping(&mut self, mapping: SensorMapping) {
        self.mappings.push(mapping);
    }

    /// Latest reading for a sensor, if any.
    pub fn latest_for(&self, sensor_id: &str) -> Option<&SensorReading> {
        self.readings
            .iter()
            .rev()
            .find(|r| r.sensor_id == sensor_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    #[test]
    fn twin_state_latest_reading() {
        let mut state = TwinState::new();
        let asset_id = IdFactory::asset();
        state.push_reading(SensorReading::new("T-1", "2026-01-01T00:00:00Z", 22.5, "C"));
        state.push_reading(SensorReading::new("T-1", "2026-01-01T01:00:00Z", 23.0, "C"));
        state.add_mapping(SensorMapping::new(asset_id).add_sensor("T-1"));
        let latest = state.latest_for("T-1").unwrap();
        assert_eq!(latest.value, 23.0);
    }
}
