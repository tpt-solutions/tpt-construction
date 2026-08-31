// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compliance events tracked against a contract.

use serde::{Deserialize, Serialize};

/// Status of a compliance event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// Outstanding.
    Open,
    /// Satisfied.
    Satisfied,
    /// Past its due date and not satisfied.
    Overdue,
}

/// A compliance event: a permit, inspection, or obligation with a due date.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceEvent {
    /// External reference / id.
    pub id: String,
    /// Description of the obligation.
    pub description: String,
    /// Due date (RFC 3339 date).
    pub due: String,
    /// Status.
    pub status: ComplianceStatus,
}

impl ComplianceEvent {
    /// Create an open compliance event due on `due`.
    pub fn new(
        id: impl Into<String>,
        description: impl Into<String>,
        due: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            due: due.into(),
            status: ComplianceStatus::Open,
        }
    }

    /// Mark satisfied.
    pub fn satisfy(&mut self) {
        self.status = ComplianceStatus::Satisfied;
    }

    /// Mark overdue.
    pub fn mark_overdue(&mut self) {
        self.status = ComplianceStatus::Overdue;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compliance_lifecycle() {
        let mut e = ComplianceEvent::new("PERMIT-1", "Building permit", "2026-02-01");
        assert_eq!(e.status, ComplianceStatus::Open);
        e.satisfy();
        assert_eq!(e.status, ComplianceStatus::Satisfied);
    }
}
