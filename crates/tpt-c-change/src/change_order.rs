// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Change orders and their review state machine.

use serde::{Deserialize, Serialize};
use tpt_c_core::ChangeOrderId;
use tpt_c_cost::Money;

use crate::ChangeError;

/// Lifecycle status of a [`ChangeOrder`].
///
/// ```text
/// Draft → Submitted → UnderReview → Approved → Implemented
///                                ↘ Rejected
/// (Submitted/UnderReview may also be Withdrawn)
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOrderStatus {
    /// Being drafted; not yet visible to the reviewer.
    Draft,
    /// Submitted for review.
    Submitted,
    /// Under active review.
    UnderReview,
    /// Approved by the reviewing party.
    Approved,
    /// Rejected by the reviewing party.
    Rejected,
    /// Approved work incorporated into the schedule/budget.
    Implemented,
    /// Withdrawn by the originator.
    Withdrawn,
}

impl ChangeOrderStatus {
    /// Whether this status allows no further transitions.
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            ChangeOrderStatus::Rejected
                | ChangeOrderStatus::Implemented
                | ChangeOrderStatus::Withdrawn
        )
    }

    /// Whether the change has been approved (or already implemented).
    pub fn is_approved(self) -> bool {
        matches!(
            self,
            ChangeOrderStatus::Approved | ChangeOrderStatus::Implemented
        )
    }

    fn can_transition(self, to: ChangeOrderStatus) -> bool {
        use ChangeOrderStatus::*;
        matches!(
            (self, to),
            (Draft, Submitted)
                | (Draft, Withdrawn)
                | (Submitted, UnderReview)
                | (Submitted, Withdrawn)
                | (UnderReview, Approved)
                | (UnderReview, Rejected)
                | (UnderReview, Withdrawn)
                | (Approved, Implemented)
        )
    }
}

/// One line of scope that needs to be re-measured for a change.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeLine {
    /// Short scope description (e.g. `"6in slab thickening"`).
    pub description: String,
    /// Affected cost code (e.g. MasterFormat `03 30 00`).
    pub cost_code: String,
    /// Cost impact attributed to this line.
    pub cost: Money,
}

/// A proposed change to the contract scope, with cost and time impact.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeOrder {
    /// Unique identifier.
    pub id: ChangeOrderId,
    /// Human-facing number (e.g. `CO-004`), unique within a project.
    pub number: String,
    /// Short title.
    pub title: String,
    /// Detailed description of the change.
    pub description: String,
    /// Current lifecycle status.
    pub status: ChangeOrderStatus,
    /// Net cost impact (negative = credit).
    pub cost_impact: Money,
    /// Net schedule impact in calendar days (negative = earlier).
    pub time_impact_days: f64,
    /// Optional per-line breakdown of the change.
    pub lines: Vec<ChangeLine>,
}

impl ChangeOrder {
    /// Create a draft change order with zero cost and time impact.
    pub fn new(id: ChangeOrderId, number: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id,
            number: number.into(),
            title: title.into(),
            description: String::new(),
            status: ChangeOrderStatus::Draft,
            cost_impact: Money::zero("USD"),
            time_impact_days: 0.0,
            lines: Vec::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the cost impact (and its currency for the order).
    pub fn with_cost_impact(mut self, cost: Money) -> Self {
        self.cost_impact = cost;
        self
    }

    /// Set the schedule impact in days.
    pub fn with_time_impact(mut self, days: f64) -> Self {
        self.time_impact_days = days;
        self
    }

    /// Append a costed line item.
    pub fn add_line(&mut self, line: ChangeLine) {
        self.lines.push(line);
    }

    /// Move the order to `to`, validating the transition.
    pub fn transition(&mut self, to: ChangeOrderStatus) -> Result<(), ChangeError> {
        if !self.status.can_transition(to) {
            return Err(ChangeError::InvalidTransition {
                from: status_name(self.status),
                to: status_name(to),
            });
        }
        self.status = to;
        Ok(())
    }
}

/// Stable names for error messages (avoids `Debug` noise).
fn status_name(status: ChangeOrderStatus) -> &'static str {
    match status {
        ChangeOrderStatus::Draft => "Draft",
        ChangeOrderStatus::Submitted => "Submitted",
        ChangeOrderStatus::UnderReview => "UnderReview",
        ChangeOrderStatus::Approved => "Approved",
        ChangeOrderStatus::Rejected => "Rejected",
        ChangeOrderStatus::Implemented => "Implemented",
        ChangeOrderStatus::Withdrawn => "Withdrawn",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> ChangeOrder {
        ChangeOrder::new(ChangeOrderId::nil(), "CO-1", "Test change")
    }

    #[test]
    fn happy_path_reaches_implemented() {
        let mut co = draft();
        assert_eq!(co.status, ChangeOrderStatus::Draft);
        co.transition(ChangeOrderStatus::Submitted).unwrap();
        co.transition(ChangeOrderStatus::UnderReview).unwrap();
        co.transition(ChangeOrderStatus::Approved).unwrap();
        co.transition(ChangeOrderStatus::Implemented).unwrap();
        assert!(co.status.is_terminal());
        assert!(co.status.is_approved());
    }

    #[test]
    fn rejection_is_terminal() {
        let mut co = draft();
        co.transition(ChangeOrderStatus::Submitted).unwrap();
        co.transition(ChangeOrderStatus::UnderReview).unwrap();
        co.transition(ChangeOrderStatus::Rejected).unwrap();
        assert!(co.status.is_terminal());
        assert!(!co.status.is_approved());
        assert!(co.transition(ChangeOrderStatus::Draft).is_err());
    }

    #[test]
    fn skips_are_rejected() {
        let mut co = draft();
        assert!(co.transition(ChangeOrderStatus::Approved).is_err());
        assert!(co.transition(ChangeOrderStatus::Implemented).is_err());
        // Draft -> Withdrawn is allowed, but the order is then closed.
        co.transition(ChangeOrderStatus::Withdrawn).unwrap();
        assert!(co.transition(ChangeOrderStatus::Submitted).is_err());
    }

    #[test]
    fn lines_accumulate() {
        let mut co = draft();
        co.add_line(ChangeLine {
            description: "Additional rebar".into(),
            cost_code: "03 20 00".into(),
            cost: Money::new(10_000.0, "USD"),
        });
        co.add_line(ChangeLine {
            description: "Formwork rework".into(),
            cost_code: "03 11 00".into(),
            cost: Money::new(2_500.0, "USD"),
        });
        assert_eq!(co.lines.len(), 2);
    }
}
