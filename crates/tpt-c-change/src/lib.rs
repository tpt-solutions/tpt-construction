// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Change orders, revisions, and cost change control.
//!
//! Construction work rarely matches the original contract exactly. This crate
//! models the change-control workflow that absorbs the difference:
//!
//! - [`ChangeOrder`] tracks a proposed change (cost and time impact) through a
//!   review state machine: `Draft → Submitted → UnderReview → Approved` or
//!   `Rejected`, ending in `Implemented` or `Withdrawn`.
//! - [`RevisionDiff`] compares two revisions of the same scope and produces
//!   per-key [`QuantityDelta`]s and per-cost-code [`CostDelta`]s:
//!
//! ```text
//! Revision A concrete: 1000 CY
//! Revision B concrete: 1050 CY
//! Delta: +50 CY
//! ```
//!
//! - [`ChangeRegister`] is the per-project ledger of change orders with the
//!   approved cost/time roll-ups a project controls report needs.
//!
//! Monetary math delegates to [`tpt_c_cost::Money`], so currency mismatches
//! are rejected instead of silently combined.

use thiserror::Error;

mod change_order;
mod diff;
mod register;

pub use change_order::{ChangeOrder, ChangeOrderStatus};
pub use diff::{CostDelta, DeltaKind, QuantityDelta, QuantityDiff, QuantityEntry, RevisionDiff};
pub use register::ChangeRegister;

/// Errors raised by change-control operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ChangeError {
    /// A status transition that the change-order state machine does not allow.
    #[error("invalid transition from {from:?} to {to:?}")]
    InvalidTransition {
        /// Current status.
        from: &'static str,
        /// Attempted target status.
        to: &'static str,
    },
    /// A change order with this number already exists in the register.
    #[error("duplicate change order number: {0}")]
    DuplicateNumber(String),
    /// A change order with this id already exists in the register.
    #[error("duplicate change order id: {0}")]
    DuplicateId(String),
    /// The register does not contain a change order with this id.
    #[error("unknown change order: {0}")]
    UnknownChangeOrder(String),
    /// Amounts in different currencies were combined.
    #[error("currency mismatch")]
    CurrencyMismatch,
}

impl From<tpt_c_cost::CostError> for ChangeError {
    fn from(e: tpt_c_cost::CostError) -> Self {
        match e {
            tpt_c_cost::CostError::CurrencyMismatch { .. } => ChangeError::CurrencyMismatch,
            _ => ChangeError::CurrencyMismatch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tpt_c_core::{ChangeOrderId, ProjectId};
    use tpt_c_cost::Money;
    use tpt_c_ids::IdFactory;

    #[test]
    fn end_to_end_change_control() {
        // Track an approved change order on a project register.
        let mut reg = ChangeRegister::new(ProjectId::nil());
        let id = ChangeOrderId::from_uuid(IdFactory::deterministic("co-1"));
        let mut co = ChangeOrder::new(id, "CO-1", "Slab thickening")
            .with_description("Owner added a mezzanine load.")
            .with_cost_impact(Money::new(18_500.0, "USD"))
            .with_time_impact(3.0);
        co.transition(ChangeOrderStatus::Submitted).unwrap();
        co.transition(ChangeOrderStatus::UnderReview).unwrap();
        co.transition(ChangeOrderStatus::Approved).unwrap();
        reg.add(co).unwrap();
        assert_eq!(reg.approved_cost_impact().unwrap().amount(), 18_500.0);
        assert_eq!(reg.approved_time_impact_days(), 3.0);

        // The spec §12 example: 1000 CY -> 1050 CY is a +50 CY delta.
        let mut a = BTreeMap::new();
        a.insert("concrete".to_string(), QuantityEntry::new("CY", 1000.0));
        let mut b = BTreeMap::new();
        b.insert("concrete".to_string(), QuantityEntry::new("CY", 1050.0));
        let diff =
            RevisionDiff::compute("rev-A", "rev-B", &a, &b, &BTreeMap::new(), &BTreeMap::new());
        assert_eq!(diff.quantities.deltas[0].delta(), 50.0);
        assert_eq!(diff.net_cost_delta().unwrap().amount(), 0.0);
    }
}
