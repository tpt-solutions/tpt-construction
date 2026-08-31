// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Contracts and contract administration.
//!
//! A [`Contract`] groups the parties, line [`ContractItem`]s, and the
//! administrative events that surround it: formal [`Notice`]s (driven by a
//! [`NoticeWorkflow`](tpt_c_workflow::NoticeWorkflow)), [`Claim`]s, and
//! [`ComplianceEvent`]s. Amounts are kept as `(f64, currency)` pairs so the
//! crate stays free of the cost-model dependency; partner crates can map these
//! onto [`tpt_c_cost::Money`] when pricing.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tpt_c_core::CoreError;

mod claim;
mod compliance;
mod contract;
mod notice;

pub use claim::{Claim, ClaimKind, ClaimStatus};
pub use compliance::{ComplianceEvent, ComplianceStatus};
pub use contract::{Contract, ContractItem, Party, Responsibility};
pub use notice::Notice;

/// Errors raised by contract operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContractError {
    /// A referenced entity was not found.
    #[error("not found: {0}")]
    NotFound(&'static str),
    /// A numeric value was out of range.
    #[error("value out of range: {0}")]
    OutOfRange(&'static str),
    /// A core domain error occurred.
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// A monetary amount with an ISO 4217 currency code.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Amount {
    /// Numeric value.
    pub value: f64,
    /// Currency code (e.g. "USD").
    pub currency: String,
}

impl Amount {
    /// Create an amount.
    pub fn new(value: f64, currency: impl Into<String>) -> Self {
        Self {
            value,
            currency: currency.into(),
        }
    }

    /// Sum two amounts of the same currency.
    pub fn checked_add(self, other: Amount) -> Result<Amount, ContractError> {
        if self.currency != other.currency {
            return Err(ContractError::OutOfRange("currency mismatch"));
        }
        Ok(Amount::new(self.value + other.value, self.currency))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::AuditMeta;
    use tpt_c_ids::{ContractId, NoticeId};
    use uuid::Uuid;

    #[test]
    fn contract_total_value() {
        let mut c = Contract::new(ContractId::from_uuid(Uuid::now_v7()), "GC Agreement");
        c.add_item("Concrete", Amount::new(100_000.0, "USD"));
        c.add_item("Steel", Amount::new(250_000.0, "USD"));
        let total = c.total_value().unwrap();
        assert_eq!(total.value, 350_000.0);
    }

    #[test]
    fn notice_drives_state() {
        let mut n = Notice::new(NoticeId::from_uuid(Uuid::now_v7()), "Notice of delay");
        n.workflow
            .acknowledge(AuditMeta::new("owner", "2026-01-01T00:00:00Z"))
            .unwrap();
        assert_eq!(
            n.workflow.state(),
            tpt_c_workflow::NoticeState::Acknowledged
        );
    }

    #[test]
    fn amount_currency_mismatch() {
        let a = Amount::new(1.0, "USD");
        let b = Amount::new(1.0, "EUR");
        assert!(a.checked_add(b).is_err());
    }
}
