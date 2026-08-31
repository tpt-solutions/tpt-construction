// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Inspections.

use serde::{Deserialize, Serialize};

/// The result of an inspection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionResult {
    /// Passed.
    Pass,
    /// Passed with noted deficiencies.
    PassWithNotes,
    /// Failed.
    Fail,
}

/// An inspection of a system, area, or activity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inspection {
    /// Identifier / reference.
    pub id: String,
    /// Subject inspected.
    pub subject: String,
    /// Date of inspection (RFC 3339 date).
    pub date: String,
    /// Result.
    pub result: InspectionResult,
    /// Optional note (deficiencies, comments).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Inspection {
    /// Create an inspection with a result.
    pub fn new(
        id: impl Into<String>,
        subject: impl Into<String>,
        date: impl Into<String>,
        result: InspectionResult,
    ) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            date: date.into(),
            result,
            note: None,
        }
    }

    /// Attach a note.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Whether the inspection passed (with or without notes).
    pub fn passed(&self) -> bool {
        matches!(
            self.result,
            InspectionResult::Pass | InspectionResult::PassWithNotes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passed_variants() {
        let pass = Inspection::new("I-1", "Scaffold", "2026-03-03", InspectionResult::Pass);
        let fail = Inspection::new("I-2", "Scaffold", "2026-03-03", InspectionResult::Fail);
        assert!(pass.passed());
        assert!(!fail.passed());
    }
}
