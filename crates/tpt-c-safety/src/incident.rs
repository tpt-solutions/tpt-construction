// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Incidents and their severity.

use serde::{Deserialize, Serialize};
use tpt_c_ids::SafetyIncidentId;

/// Severity of a safety event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Minor / first-aid.
    Low,
    /// Recordable.
    Medium,
    /// Lost-time / serious.
    High,
    /// Fatal or catastrophic.
    Critical,
}

/// Lifecycle status of an incident.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    /// Reported, not yet investigated.
    Open,
    /// Under investigation.
    Investigating,
    /// Closed / resolved.
    Closed,
}

/// A safety incident.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Incident {
    /// Incident identifier.
    pub id: SafetyIncidentId,
    /// Description.
    pub description: String,
    /// Severity.
    pub severity: Severity,
    /// Whether an injury occurred.
    #[serde(default)]
    pub injury: bool,
    /// Status.
    pub status: IncidentStatus,
    /// Optional root-cause / outcome note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
}

impl Incident {
    /// Create an open incident.
    pub fn new(id: SafetyIncidentId, description: impl Into<String>, severity: Severity) -> Self {
        Self {
            id,
            description: description.into(),
            severity,
            injury: false,
            status: IncidentStatus::Open,
            outcome: None,
        }
    }

    /// Mark the incident under investigation.
    pub fn begin_investigation(&mut self) {
        self.status = IncidentStatus::Investigating;
    }

    /// Close the incident with an optional outcome note.
    pub fn close(&mut self) {
        self.status = IncidentStatus::Closed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::SafetyIncidentId;

    #[test]
    fn injury_flag() {
        let mut i = Incident::new(SafetyIncidentId::from_uuid(uuid::Uuid::now_v7()), "Laceration", Severity::Low);
        i.injury = true;
        i.begin_investigation();
        assert_eq!(i.status, IncidentStatus::Investigating);
    }
}
