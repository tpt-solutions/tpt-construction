// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Facility management: COBie handover data, asset registers, spaces and systems.
//!
//! Provides types for handover data exchange (COBie-style), facility asset
//! registers, and the space/system taxonomy used in FM contexts.

use serde::{Deserialize, Serialize};
use tpt_c_core::AssetId;

/// A facility asset entry for handover registers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacilityAsset {
    /// Asset identifier.
    pub id: AssetId,
    /// Human-readable name.
    pub name: String,
    /// System this asset belongs to (e.g. HVAC, electrical).
    pub system: String,
    /// Space / zone where the asset is installed.
    pub space: String,
    /// Manufacturer name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    /// Model number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Serial number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
}

impl FacilityAsset {
    /// Build a facility asset record.
    pub fn new(
        id: AssetId,
        name: impl Into<String>,
        system: impl Into<String>,
        space: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            system: system.into(),
            space: space.into(),
            manufacturer: None,
            model: None,
            serial_number: None,
        }
    }

    /// Attach manufacturer/model info.
    pub fn with_manufacturer(
        mut self,
        manufacturer: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        self.manufacturer = Some(manufacturer.into());
        self.model = Some(model.into());
        self
    }

    /// Attach a serial number.
    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        self.serial_number = Some(serial.into());
        self
    }
}

/// A COBie-style component record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CobieComponent {
    /// Component name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Space where the component is installed.
    pub space: String,
    /// System name.
    pub system: String,
    /// Manufacturer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    /// Model number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_number: Option<String>,
}

impl CobieComponent {
    /// Build a COBie component record.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        space: impl Into<String>,
        system: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            space: space.into(),
            system: system.into(),
            manufacturer: None,
            model_number: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    #[test]
    fn facility_asset_builder() {
        let id = IdFactory::asset();
        let asset = FacilityAsset::new(id, "AHU-1", "HVAC", "Level 1")
            .with_manufacturer("Trane", "S Helix")
            .with_serial("SN12345");
        assert_eq!(asset.system, "HVAC");
        assert_eq!(asset.space, "Level 1");
        assert_eq!(asset.manufacturer, Some("Trane".to_string()));
    }

    #[test]
    fn cobie_component_roundtrip() {
        let comp = CobieComponent::new("VAV-1", "Variable air volume box", "Room 101", "HVAC");
        let json = serde_json::to_string(&comp).unwrap();
        let back: CobieComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(comp, back);
    }
}
