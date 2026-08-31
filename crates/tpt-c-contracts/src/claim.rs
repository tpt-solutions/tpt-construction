// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Claims: delay, disruption, acceleration, or payment.

use serde::{Deserialize, Serialize};
use tpt_c_ids::ClaimId;

use crate::Amount;

/// The kind of claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    /// Time extension / delay.
    Delay,
    /// Disruption to productivity.
    Disruption,
    /// Acceleration costs.
    Acceleration,
    /// Payment / valuation.
    Payment,
    /// Other.
    Other,
}

/// Lifecycle status of a claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    /// Raised.
    Open,
    /// Submitted for review.
    Submitted,
    /// Under review.
    UnderReview,
    /// Resolved (settled).
    Resolved,
    /// Rejected.
    Rejected,
}

/// A claim raised under a contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    /// Claim identifier.
    pub id: ClaimId,
    /// Claim kind.
    pub kind: ClaimKind,
    /// Description.
    pub description: String,
    /// Claimed amount, if quantified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    /// Status.
    pub status: ClaimStatus,
}

impl Claim {
    /// Create an open claim.
    pub fn new(id: ClaimId, kind: ClaimKind, description: impl Into<String>) -> Self {
        Self {
            id,
            kind,
            description: description.into(),
            amount: None,
            status: ClaimStatus::Open,
        }
    }

    /// Attach a claimed amount.
    pub fn with_amount(mut self, amount: Amount) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Submit the claim.
    pub fn submit(&mut self) {
        self.status = ClaimStatus::Submitted;
    }

    /// Resolve the claim.
    pub fn resolve(&mut self) {
        self.status = ClaimStatus::Resolved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_submit_and_resolve() {
        let mut c = Claim::new(
            ClaimId::from_uuid(uuid::Uuid::now_v7()),
            ClaimKind::Delay,
            "Weather delay",
        );
        assert_eq!(c.status, ClaimStatus::Open);
        c.submit();
        c.resolve();
        assert_eq!(c.status, ClaimStatus::Resolved);
    }
}
