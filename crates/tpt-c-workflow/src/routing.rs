// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Routing metadata: who must act, when it is due, and escalation rules.
//!
//! A workflow state is only meaningful when someone owns it. [`Assignment`]
//! pairs a [`Role`] with a concrete actor (user id, vendor, reviewer) and an
//! optional [`DueDate`]. When a step blows past its due date an [`Escalation`]
//! describes who should be notified and after how long.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tpt_c_core::AuditMeta;

/// A participant role in a workflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Originator of the item (contractor, designer, owner).
    Initiator,
    /// Reviewer who must respond (architect, engineer, owner).
    Reviewer,
    /// Approver with authority to close the loop.
    Approver,
    /// Party that receives the result (often the owner or GC).
    Recipient,
    /// Distribution list / CC.
    Observer,
}

/// A due date expressed as an RFC 3339 timestamp.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DueDate(pub String);

impl DueDate {
    /// Build a due date from an RFC 3339 string. Callers are responsible for
    /// passing a parseable value; use [`DueDate::parse`] for validation.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Parse and validate the RFC 3339 timestamp.
    pub fn parse(&self) -> Result<chrono_like::Parsed, RoutingError> {
        chrono_like::parse_rfc3339(&self.0)
    }
}

/// Escalation triggered when an assignment is overdue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Escalation {
    /// Role to notify on escalation.
    pub notify: Role,
    /// Actor (user id / distribution) to notify.
    pub actor: String,
    /// Overdue threshold in calendar days before escalation fires.
    pub after_days: u32,
}

/// An assignment of a role to an actor for the lifetime of a workflow step.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    /// The role being filled.
    pub role: Role,
    /// Concrete actor (user id, vendor code, email, ...).
    pub actor: String,
    /// Optional due date for this assignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<DueDate>,
    /// Optional escalation when the due date is missed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub escalation: Option<Escalation>,
}

impl Assignment {
    /// Create an assignment for a role and actor.
    pub fn new(role: Role, actor: impl Into<String>) -> Self {
        Self {
            role,
            actor: actor.into(),
            due: None,
            escalation: None,
        }
    }

    /// Attach a due date.
    pub fn with_due(mut self, due: DueDate) -> Self {
        self.due = Some(due);
        self
    }

    /// Attach an escalation rule.
    pub fn with_escalation(mut self, escalation: Escalation) -> Self {
        self.escalation = Some(escalation);
        self
    }

    /// Record that this assignment was satisfied at `at` by `actor`.
    pub fn satisfied_by(&self, at: AuditMeta) -> AssignmentRecord {
        AssignmentRecord {
            role: self.role,
            actor: self.actor.clone(),
            at,
        }
    }
}

/// A record that an assignment was completed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentRecord {
    /// Role that was satisfied.
    pub role: Role,
    /// Actor that satisfied it.
    pub actor: String,
    /// When it was satisfied.
    pub at: AuditMeta,
}

/// Errors raised while building routing metadata.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RoutingError {
    /// A supplied RFC 3339 timestamp could not be parsed.
    #[error("invalid due date timestamp: {0}")]
    InvalidDueDate(String),
}

/// Minimal RFC 3339 parsing helper kept local to avoid a chrono dependency.
mod chrono_like {
    use super::RoutingError;

    /// A successfully parsed timestamp (we only validate structure here).
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Parsed {
        /// The raw validated string.
        pub raw: String,
    }

    /// Validate that `s` looks like an RFC 3339 timestamp. Full date/time
    /// parsing is out of scope; we require `YYYY-MM-DD` followed by `T` and a
    /// time component, optionally with a `Z` or offset.
    pub fn parse_rfc3339(s: &str) -> Result<Parsed, RoutingError> {
        let (date, time) = s
            .split_once('T')
            .ok_or_else(|| RoutingError::InvalidDueDate(s.into()))?;
        let date_ok = date.len() == 10 && date.bytes().all(|b| b.is_ascii_digit() || b == b'-');
        let start = time
            .trim_end_matches('Z')
            .split(['+', '-'])
            .next()
            .unwrap_or("");
        let time_ok = start.len() >= 8 && start.bytes().all(|b| b.is_ascii_digit() || b == b':');
        if date_ok && time_ok {
            Ok(Parsed { raw: s.into() })
        } else {
            Err(RoutingError::InvalidDueDate(s.into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_builder_and_due_validation() {
        let a = Assignment::new(Role::Reviewer, "arch@example.com")
            .with_due(DueDate::new("2026-02-01T17:00:00Z"));
        assert_eq!(a.role, Role::Reviewer);
        assert!(a.due.is_some());
        assert!(a.due.unwrap().parse().is_ok());
    }

    #[test]
    fn rejects_malformed_due_date() {
        let a = Assignment::new(Role::Reviewer, "x").with_due(DueDate::new("soon"));
        assert!(a.due.unwrap().parse().is_err());
    }
}
