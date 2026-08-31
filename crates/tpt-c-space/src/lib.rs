// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spaces, occupancy, leases, areas, space planning, space utilization.
//!
//! Provides the space inventory model used by FM and digital twin crates:
//! sites, floors, spaces, occupancy, and lease metadata.

use serde::{Deserialize, Serialize};
use tpt_c_core::ProjectId;

/// A space within a building or site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Space {
    /// Stable space identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Category (office, corridor, mechanical, ...).
    pub category: String,
    /// Net area in square metres.
    pub net_area: f64,
    /// Gross area in square metres, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gross_area: Option<f64>,
    /// Floor / level name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// Occupancy count, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<u32>,
}

impl Space {
    /// Build a space.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        category: impl Into<String>,
        net_area: f64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            category: category.into(),
            net_area,
            gross_area: None,
            floor: None,
            occupancy: None,
        }
    }

    /// Set gross area.
    pub fn with_gross_area(mut self, gross_area: f64) -> Self {
        self.gross_area = Some(gross_area);
        self
    }

    /// Set the floor / level.
    pub fn with_floor(mut self, floor: impl Into<String>) -> Self {
        self.floor = Some(floor.into());
        self
    }

    /// Set occupancy.
    pub fn with_occupancy(mut self, occupancy: u32) -> Self {
        self.occupancy = Some(occupancy);
        self
    }
}

/// A lease tied to a space.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lease {
    /// Lease identifier.
    pub id: String,
    /// Tenant name.
    pub tenant: String,
    /// ISO 8601 start date.
    pub start_date: String,
    /// ISO 8601 end date.
    pub end_date: String,
    /// Net lettable area in square metres.
    pub area: f64,
}

impl Lease {
    /// Build a lease.
    pub fn new(
        id: impl Into<String>,
        tenant: impl Into<String>,
        start_date: impl Into<String>,
        end_date: impl Into<String>,
        area: f64,
    ) -> Self {
        Self {
            id: id.into(),
            tenant: tenant.into(),
            start_date: start_date.into(),
            end_date: end_date.into(),
            area,
        }
    }
}

/// A space inventory for a project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpaceInventory {
    /// Project this inventory belongs to.
    pub project_id: ProjectId,
    /// Spaces keyed by id.
    pub spaces: Vec<Space>,
    /// Leases keyed by id.
    pub leases: Vec<Lease>,
}

impl Default for SpaceInventory {
    fn default() -> Self {
        Self {
            project_id: ProjectId::nil(),
            spaces: Vec::new(),
            leases: Vec::new(),
        }
    }
}

impl SpaceInventory {
    /// Build an empty inventory for a project.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            spaces: Vec::new(),
            leases: Vec::new(),
        }
    }

    /// Add a space.
    pub fn add_space(&mut self, space: Space) {
        self.spaces.push(space);
    }

    /// Add a lease.
    pub fn add_lease(&mut self, lease: Lease) {
        self.leases.push(lease);
    }

    /// Total net area across all spaces.
    pub fn total_net_area(&self) -> f64 {
        self.spaces.iter().map(|s| s.net_area).sum()
    }

    /// Total leased area.
    pub fn total_leased_area(&self) -> f64 {
        self.leases.iter().map(|l| l.area).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_inventory_totals() {
        let project_id = ProjectId::nil();
        let mut inv = SpaceInventory::new(project_id);
        inv.add_space(Space::new("S1", "Office A", "office", 25.0).with_gross_area(30.0));
        inv.add_space(Space::new("S2", "Office B", "office", 35.0));
        inv.add_lease(Lease::new(
            "L1",
            "Acme Corp",
            "2026-01-01",
            "2027-01-01",
            25.0,
        ));
        assert_eq!(inv.total_net_area(), 60.0);
        assert_eq!(inv.total_leased_area(), 25.0);
    }
}
