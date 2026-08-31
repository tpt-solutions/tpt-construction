// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Site observations raised in the field (safety, quality, environmental, ...).

use serde::{Deserialize, Serialize};
use tpt_c_core::AuditMeta;
use uuid::Uuid;

/// Category of a site observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationCategory {
    /// Worker safety concern.
    Safety,
    /// Conformance / quality issue.
    Quality,
    /// Environmental impact or compliance.
    Environmental,
    /// General note.
    General,
}

/// Severity of a site observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Minor.
    Low,
    /// Notable.
    Medium,
    /// Serious.
    High,
    /// Critical / stop-work.
    Critical,
}

/// Lifecycle status of a site observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    /// Raised, not yet reviewed.
    Open,
    /// Under review.
    InReview,
    /// Resolved / closed.
    Closed,
}

/// A site observation captured in the field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteObservation {
    /// Observation identifier.
    pub id: ObservationId,
    /// Category.
    pub category: ObservationCategory,
    /// Description.
    pub description: String,
    /// Severity.
    pub severity: Severity,
    /// Current status.
    pub status: ObservationStatus,
    /// Optional resolution note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    /// Who raised it.
    pub raised_by: String,
    /// When it was raised.
    pub raised_at: AuditMeta,
    /// When it was resolved (if closed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<AuditMeta>,
}

/// Identifier for a site observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ObservationId(Uuid);

impl ObservationId {
    /// Build from an explicit UUID.
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }
    /// The nil identifier.
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl SiteObservation {
    /// Raise a new observation.
    pub fn new(
        category: ObservationCategory,
        description: impl Into<String>,
        severity: Severity,
    ) -> Self {
        Self {
            id: ObservationId::from_uuid(Uuid::now_v7()),
            category,
            description: description.into(),
            severity,
            status: ObservationStatus::Open,
            resolution: None,
            raised_by: "field".to_string(),
            raised_at: AuditMeta::new("field", "1970-01-01T00:00:00Z"),
            resolved_at: None,
        }
    }

    /// Mark the observation under review.
    pub fn review(&mut self, at: AuditMeta) {
        self.status = ObservationStatus::InReview;
        self.raised_at = at;
    }

    /// Resolve (close) the observation.
    pub fn resolve(&mut self, at: AuditMeta) {
        self.status = ObservationStatus::Closed;
        self.resolved_at = Some(at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::AuditMeta;

    #[test]
    fn severity_ordering() {
        assert!(Severity::Critical > Severity::Low);
    }

    #[test]
    fn review_then_resolve() {
        let mut o = SiteObservation::new(ObservationCategory::Quality, "Honeycombing", Severity::Medium);
        o.review(AuditMeta::new("qa", "2026-01-01T00:00:00Z"));
        assert_eq!(o.status, ObservationStatus::InReview);
        o.resolve(AuditMeta::new("qa", "2026-01-02T00:00:00Z"));
        assert_eq!(o.status, ObservationStatus::Closed);
        assert!(o.resolved_at.is_some());
    }
}
