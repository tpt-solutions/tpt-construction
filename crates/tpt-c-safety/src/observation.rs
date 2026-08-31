// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Near misses and safety observations.

use serde::{Deserialize, Serialize};

/// Category of a safety observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyCategory {
    /// Housekeeping / slip-trip-fall.
    Housekeeping,
    /// Fall from height.
    Height,
    /// Electrical.
    Electrical,
    /// Plant / equipment.
    Plant,
    /// Environmental.
    Environmental,
    /// Other.
    Other,
}

/// A near miss (an unplanned event with no injury / damage).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NearMiss {
    /// Identifier.
    pub id: String,
    /// Description.
    pub description: String,
    /// Category.
    pub category: SafetyCategory,
    /// Who reported it.
    pub reported_by: String,
    /// Whether it has been reviewed.
    #[serde(default)]
    pub reviewed: bool,
}

impl NearMiss {
    /// Create a near miss.
    pub fn new(id: impl Into<String>, description: impl Into<String>, category: SafetyCategory, reported_by: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            category,
            reported_by: reported_by.into(),
            reviewed: false,
        }
    }

    /// Mark reviewed.
    pub fn review(&mut self) {
        self.reviewed = true;
    }
}

/// A safety observation raised on site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SafetyObservation {
    /// Identifier.
    pub id: String,
    /// Description.
    pub description: String,
    /// Category.
    pub category: SafetyCategory,
    /// Whether a corrective action was taken.
    #[serde(default)]
    pub actioned: bool,
}

impl SafetyObservation {
    /// Create a safety observation.
    pub fn new(id: impl Into<String>, description: impl Into<String>, category: SafetyCategory) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            category,
            actioned: false,
        }
    }

    /// Mark actioned.
    pub fn action(&mut self) {
        self.actioned = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn near_miss_review() {
        let mut n = NearMiss::new("NM-1", "Material fell near worker", SafetyCategory::Height, "carlos");
        assert!(!n.reviewed);
        n.review();
        assert!(n.reviewed);
    }
}
