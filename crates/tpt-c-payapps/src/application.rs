// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Payment applications (progress claims).

use serde::{Deserialize, Serialize};
use tpt_c_cost::Money;
use tpt_c_ids::PaymentApplicationId;

use crate::PayAppError;

/// Lifecycle status of a payment application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentApplicationStatus {
    /// Drafted by the contractor.
    Draft,
    /// Submitted for review.
    Submitted,
    /// Under review by the owner / engineer.
    UnderReview,
    /// Approved.
    Approved,
    /// Rejected.
    Rejected,
    /// Certified (payable).
    Certified,
}

/// A payment application drawing down against a contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentApplication {
    /// Application identifier.
    pub id: PaymentApplicationId,
    /// Billing period (e.g. "2026-03").
    pub period: String,
    /// Schedule of values this application draws against.
    pub sov_id: String,
    /// Current status.
    pub status: PaymentApplicationStatus,
    /// Value of work completed to date (excluding stored materials).
    pub work_completed: Money,
    /// Value of stored materials on site.
    pub stored_materials: Money,
    /// Retainage rate as a fraction (e.g. 0.05 for 5%).
    #[serde(default)]
    pub retainage: f64,
    /// Amount certified in all prior applications.
    pub previously_certified: Money,
    /// Who approved the application.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
}

impl PaymentApplication {
    /// Create a draft application for `period` against `sov_total`.
    ///
    /// `sov_total` seeds `previously_certified` to a zero amount of the same
    /// currency so subsequent arithmetic is currency-safe.
    pub fn new(id: PaymentApplicationId, period: impl Into<String>, sov_total: Money) -> Self {
        let currency = sov_total.currency().to_string();
        Self {
            id,
            period: period.into(),
            sov_id: String::new(),
            status: PaymentApplicationStatus::Draft,
            work_completed: Money::zero(currency.clone()),
            stored_materials: Money::zero(currency.clone()),
            retainage: 0.0,
            previously_certified: Money::zero(currency),
            approved_by: None,
        }
    }

    /// Set the schedule of values reference.
    pub fn with_sov(mut self, sov_id: impl Into<String>) -> Self {
        self.sov_id = sov_id.into();
        self
    }

    /// Set work completed to date.
    pub fn set_work_completed(&mut self, amount: Money) {
        self.work_completed = amount;
    }

    /// Set stored materials value.
    pub fn set_stored_materials(&mut self, amount: Money) {
        self.stored_materials = amount;
    }

    /// Set retainage as a fraction in `[0, 1]`.
    pub fn set_retainage(&mut self, fraction: f64) -> Result<(), PayAppError> {
        if !(0.0..=1.0).contains(&fraction) {
            return Err(PayAppError::OutOfRange("retainage"));
        }
        self.retainage = fraction;
        Ok(())
    }

    /// Set the amount previously certified.
    pub fn set_previously_certified(&mut self, amount: Money) {
        self.previously_certified = amount;
    }

    /// Gross valuation this period: work completed plus stored materials.
    pub fn gross_val(&self) -> Money {
        self.work_completed.clone() + self.stored_materials.clone()
    }

    /// Retainage held this period (gross valuation * retainage).
    pub fn retainage_amount(&self) -> Money {
        self.gross_val() * self.retainage
    }

    /// Net amount payable this period.
    ///
    /// `gross - retainage - previously_certified`.
    pub fn net_claim(&self) -> Money {
        let gross = self.gross_val();
        let retained = gross.clone() * self.retainage;
        (gross - retained) - self.previously_certified.clone()
    }

    /// Submit the application.
    pub fn submit(&mut self) {
        self.status = PaymentApplicationStatus::Submitted;
    }

    /// Move to under review.
    pub fn begin_review(&mut self) {
        self.status = PaymentApplicationStatus::UnderReview;
    }

    /// Approve the application.
    pub fn approve(&mut self, by: String) {
        self.status = PaymentApplicationStatus::Approved;
        self.approved_by = Some(by);
    }

    /// Reject the application.
    pub fn reject(&mut self) {
        self.status = PaymentApplicationStatus::Rejected;
        self.approved_by = None;
    }

    /// Certify the approved application (payable).
    pub fn certify(&mut self) {
        if self.status == PaymentApplicationStatus::Approved {
            self.status = PaymentApplicationStatus::Certified;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::PaymentApplicationId;

    #[test]
    fn gross_and_retainage() {
        let mut app = PaymentApplication::new(
            PaymentApplicationId::from_uuid(uuid::Uuid::now_v7()),
            "p",
            Money::new(1_000_000.0, "USD"),
        );
        app.set_work_completed(Money::new(500_000.0, "USD"));
        app.set_retainage(0.10).unwrap();
        assert_eq!(app.gross_val().amount(), 500_000.0);
        assert_eq!(app.retainage_amount().amount(), 50_000.0);
    }

    #[test]
    fn certify_requires_approval() {
        let mut app = PaymentApplication::new(
            PaymentApplicationId::from_uuid(uuid::Uuid::now_v7()),
            "p",
            Money::new(1.0, "USD"),
        );
        app.certify();
        assert_eq!(app.status, PaymentApplicationStatus::Draft);
        app.approve("owner".into());
        app.certify();
        assert_eq!(app.status, PaymentApplicationStatus::Certified);
    }
}
