// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Preventive and corrective maintenance, work orders, and service history.
//!
//! Models the maintenance lifecycle for assets: scheduled preventive tasks,
//! reactive corrective work orders, and a read-only service history.

use serde::{Deserialize, Serialize};
use tpt_c_core::AssetId;

/// Maintenance kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceKind {
    /// Scheduled preventive maintenance.
    Preventive,
    /// Unplanned corrective repair.
    Corrective,
    /// Inspection or condition assessment.
    Inspection,
    /// Upgrade or modification.
    Modification,
}

/// Current state of a maintenance work order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceStatus {
    /// Work order has been requested.
    Requested,
    /// Work order has been approved and scheduled.
    Scheduled,
    /// Work is in progress.
    InProgress,
    /// Work is complete and verified.
    Completed,
    /// Work order has been cancelled.
    Cancelled,
}

/// A maintenance work order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkOrder {
    /// Work order identifier.
    pub id: String,
    /// Target asset.
    pub asset_id: AssetId,
    /// Kind of maintenance.
    pub kind: MaintenanceKind,
    /// Current status.
    pub status: MaintenanceStatus,
    /// Description of the work.
    pub description: String,
    /// Service history entries (completed steps).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<ServiceEntry>,
}

impl WorkOrder {
    /// Create a new requested work order.
    pub fn new(
        id: impl Into<String>,
        asset_id: AssetId,
        kind: MaintenanceKind,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            asset_id,
            kind,
            status: MaintenanceStatus::Requested,
            description: description.into(),
            history: Vec::new(),
        }
    }

    /// Advance the work order status.
    pub fn transition(&mut self, status: MaintenanceStatus) {
        self.status = status;
    }

    /// Append a service history entry.
    pub fn add_entry(&mut self, entry: ServiceEntry) {
        self.history.push(entry);
    }
}

/// A single service history record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ServiceEntry {
    /// ISO 8601 date of the service.
    pub date: String,
    /// Who performed the work.
    pub performed_by: String,
    /// Notes about the service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ServiceEntry {
    /// Build a service entry.
    pub fn new(date: impl Into<String>, performed_by: impl Into<String>) -> Self {
        Self {
            date: date.into(),
            performed_by: performed_by.into(),
            notes: None,
        }
    }

    /// Attach notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    #[test]
    fn work_order_lifecycle() {
        let asset_id = IdFactory::asset();
        let mut wo = WorkOrder::new(
            "WO-1",
            asset_id,
            MaintenanceKind::Preventive,
            "Filter change",
        );
        assert_eq!(wo.status, MaintenanceStatus::Requested);
        wo.transition(MaintenanceStatus::InProgress);
        wo.add_entry(ServiceEntry::new("2026-03-01", "alice").with_notes("Replaced filter"));
        assert_eq!(wo.history.len(), 1);
        wo.transition(MaintenanceStatus::Completed);
        assert_eq!(wo.status, MaintenanceStatus::Completed);
    }
}
