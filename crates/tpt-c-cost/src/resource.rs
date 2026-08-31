// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Resource rates and cost codes.

use serde::{Deserialize, Serialize};

use crate::Money;

/// The kind of a priced resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    /// Direct labor.
    Labor,
    /// Material.
    Material,
    /// Equipment (owned or rented).
    Equipment,
    /// Subcontracted work.
    Subcontractor,
}

/// The unit a resource rate is expressed per.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RateUnit {
    /// Per hour.
    Hour,
    /// Per day (24 h).
    Day,
    /// Per week (168 h).
    Week,
    /// Per discrete unit ("each").
    Each,
    /// Per linear unit (metre).
    Length,
    /// Per area unit (m²).
    Area,
    /// Per volume unit (m³).
    Volume,
    /// Per mass unit (kg).
    Mass,
}

/// A unit rate for a resource, keyed for lookup by cost code.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceRate {
    /// Stable rate id.
    pub id: String,
    /// Resource kind.
    pub kind: ResourceKind,
    /// Description.
    pub description: String,
    /// Unit the rate is expressed per.
    pub unit: RateUnit,
    /// The rate amount (per one unit) in the database currency.
    pub rate: Money,
}

impl ResourceRate {
    /// Build a resource rate.
    pub fn new(id: impl Into<String>, kind: ResourceKind, unit: RateUnit, rate: Money) -> Self {
        Self {
            id: id.into(),
            kind,
            description: String::new(),
            unit,
            rate,
        }
    }

    /// Attach a description.
    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_rate_builder() {
        let r = ResourceRate::new(
            "r1",
            ResourceKind::Labor,
            RateUnit::Hour,
            Money::new(75.0, "USD"),
        )
        .with_description("Journeyman");
        assert_eq!(r.rate.amount(), 75.0);
        assert_eq!(r.description, "Journeyman");
    }
}
