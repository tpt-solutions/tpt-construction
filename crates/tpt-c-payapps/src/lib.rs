// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Progress claims and payment applications.
//!
//! A [`ScheduleOfValues`] breaks a contract into valued line items. A
//! [`PaymentApplication`] draws down against it: work completed to date plus
//! stored materials, less retainage and previously certified amounts, yields
//! the amount payable this period. Applications move through an
//! [`PaymentApplicationStatus`] as they are submitted, reviewed, approved, and
//! certified. Monetary math uses [`tpt_c_cost::Money`].

use thiserror::Error;

mod application;
mod sov;

pub use application::{PaymentApplication, PaymentApplicationStatus};
pub use sov::{ScheduleOfValues, SovItem};

/// Errors raised by payment application operations.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PayAppError {
    /// Two amounts had incompatible currencies.
    #[error("currency mismatch")]
    CurrencyMismatch,
    /// A numeric value was out of range (e.g. retainage not in 0..=1).
    #[error("value out of range: {0}")]
    OutOfRange(&'static str),
}

impl From<tpt_c_cost::CostError> for PayAppError {
    fn from(e: tpt_c_cost::CostError) -> Self {
        match e {
            tpt_c_cost::CostError::CurrencyMismatch { .. } => PayAppError::CurrencyMismatch,
            _ => PayAppError::OutOfRange("cost error"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_cost::Money;
    use tpt_c_ids::PaymentApplicationId;

    #[test]
    fn sov_total() {
        let mut sov = ScheduleOfValues::new("SOV-1");
        sov.add_item("01", "Foundations", Money::new(200_000.0, "USD"));
        sov.add_item("02", "Superstructure", Money::new(800_000.0, "USD"));
        assert_eq!(sov.total().amount(), 1_000_000.0);
    }

    #[test]
    fn application_net_claim() {
        let mut sov = ScheduleOfValues::new("SOV-1");
        sov.add_item("01", "Foundations", Money::new(200_000.0, "USD"));
        let mut app = PaymentApplication::new(
            PaymentApplicationId::from_uuid(uuid::Uuid::now_v7()),
            "2026-03",
            sov.total(),
        );
        app.set_work_completed(Money::new(150_000.0, "USD"));
        app.set_retainage(0.05);
        // gross 150k, retainage 7.5k, previously 0 => net 142.5k
        assert_eq!(app.net_claim().amount(), 142_500.0);
        app.approve("owner".to_string());
        assert_eq!(app.status, PaymentApplicationStatus::Approved);
    }

    #[test]
    fn retainage_must_be_fraction() {
        let mut sov = ScheduleOfValues::new("SOV-1");
        sov.add_item("01", "X", Money::new(100.0, "USD"));
        let mut app = PaymentApplication::new(
            PaymentApplicationId::from_uuid(uuid::Uuid::now_v7()),
            "p1",
            sov.total(),
        );
        assert!(app.set_retainage(1.5).is_err());
    }
}
