// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Safety management for construction sites.
//!
//! Captures the full safety picture: reportable [`Incident`]s, [`NearMiss`]es,
//! [`SafetyObservation`]s, [`ToolboxTalk`]s, [`Inspection`]s, and the
//! [`ComplianceChecklist`]s used to verify controls are in place. Severity and
//! status enums give each record a clear lifecycle.

use thiserror::Error;

mod checklist;
mod incident;
mod inspection;
mod observation;
mod toolbox;

pub use checklist::{ChecklistItem, ChecklistStatus, ComplianceChecklist};
pub use incident::{Incident, IncidentStatus, Severity};
pub use inspection::{Inspection, InspectionResult};
pub use observation::{NearMiss, SafetyCategory, SafetyObservation};
pub use toolbox::ToolboxTalk;

/// Errors raised by safety operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum SafetyError {
    /// A referenced record or item was not found.
    #[error("not found: {0}")]
    NotFound(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::SafetyIncidentId;
    use uuid::Uuid;

    #[test]
    fn incident_lifecycle() {
        let mut inc = Incident::new(
            SafetyIncidentId::from_uuid(Uuid::now_v7()),
            "Fall from height",
            Severity::High,
        );
        assert_eq!(inc.status, IncidentStatus::Open);
        inc.begin_investigation();
        assert_eq!(inc.status, IncidentStatus::Investigating);
        inc.close();
        assert_eq!(inc.status, IncidentStatus::Closed);
    }

    #[test]
    fn checklist_completion() {
        let mut c = ComplianceChecklist::new("Pre-task");
        c.add_item("Barricades in place");
        c.add_item("Spotter assigned");
        assert_eq!(c.open_items(), 2);
        c.items[0].mark_done();
        assert_eq!(c.open_items(), 1);
        assert!(!c.is_complete());
        c.items[1].mark_done();
        assert!(c.is_complete());
    }

    #[test]
    fn toolbox_talk_records_attendees() {
        let mut t = ToolboxTalk::new("Excavation safety", "2026-03-01");
        t.add_attendee("alice");
        t.add_attendee("bob");
        assert_eq!(t.attendees.len(), 2);
    }
}
