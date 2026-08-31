// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Asset lifecycle, warranties, serial numbers, replacements, maintenance schedules.
//!
//! Tracks physical assets from commissioning through operation to replacement,
//! including warranty windows and service intervals.

use serde::{Deserialize, Serialize};
use tpt_c_core::AssetId;

/// Lifecycle status of an asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    /// Asset is installed and operational.
    Active,
    /// Asset is temporarily out of service.
    Standby,
    /// Asset is undergoing maintenance or repair.
    UnderMaintenance,
    /// Asset has been decommissioned.
    Retired,
    /// Asset has been replaced.
    Replaced,
}

/// An asset record with lifecycle and warranty metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// Stable asset identifier.
    pub id: AssetId,
    /// Human-readable name.
    pub name: String,
    /// Current lifecycle status.
    pub status: AssetStatus,
    /// Serial number, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    /// Warranty expiry date (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warranty_expiry: Option<String>,
    /// Installation date (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_date: Option<String>,
    /// Replacement asset id, if this asset has been replaced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaced_by: Option<AssetId>,
}

impl Asset {
    /// Create a new active asset.
    pub fn new(id: AssetId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            status: AssetStatus::Active,
            serial_number: None,
            warranty_expiry: None,
            install_date: None,
            replaced_by: None,
        }
    }

    /// Attach a serial number.
    pub fn with_serial(mut self, serial: impl Into<String>) -> Self {
        self.serial_number = Some(serial.into());
        self
    }

    /// Set the warranty expiry date.
    pub fn with_warranty(mut self, expiry: impl Into<String>) -> Self {
        self.warranty_expiry = Some(expiry.into());
        self
    }

    /// Set the installation date.
    pub fn with_install_date(mut self, date: impl Into<String>) -> Self {
        self.install_date = Some(date.into());
        self
    }

    /// Mark the asset as replaced by another.
    pub fn replace(mut self, new_id: AssetId) -> Self {
        self.status = AssetStatus::Replaced;
        self.replaced_by = Some(new_id);
        self
    }

    /// True if the asset is still in active service.
    pub fn is_active(&self) -> bool {
        matches!(self.status, AssetStatus::Active | AssetStatus::Standby)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    #[test]
    fn asset_lifecycle() {
        let id = IdFactory::asset();
        let mut asset = Asset::new(id, "Chiller-1")
            .with_serial("SN999")
            .with_warranty("2028-06-01")
            .with_install_date("2026-01-15");
        assert!(asset.is_active());
        let new_id = IdFactory::asset();
        asset = asset.replace(new_id);
        assert_eq!(asset.status, AssetStatus::Replaced);
        assert_eq!(asset.replaced_by, Some(new_id));
    }
}
