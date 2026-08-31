// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Earned Value Management (EVM) for construction projects.
//!
//! Earned value integrates scope, schedule, and cost on a single basis. This
//! crate captures a periodic status snapshot — Budget at Completion ([`Amount`]),
//! Planned Value (BCWS/PV), Earned Value (BCWP/EV), and Actual Cost (ACWP/AC) —
//! and derives the standard schedule and cost performance indices, estimates at
//! completion, and to-complete performance indices.
//!
//! Monetary amounts use the crate-local [`Amount`] type (magnitude + ISO 4217
//! currency) so the crate stays free of external monetary substrates; index and
//! ratio metrics are returned as `f64`.

use serde::{Deserialize, Serialize};
use tpt_c_core::ProjectId;

/// A monetary amount in a specific ISO 4217 currency.
///
/// Arithmetic requires matching currencies; mismatches are rejected rather than
/// silently combined.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Amount {
    /// Magnitude in the major unit (e.g. dollars, not cents).
    value: f64,
    /// ISO 4217 currency code (e.g. `USD`).
    currency: String,
}

impl Amount {
    /// Build an amount in `currency`.
    pub fn new(value: f64, currency: impl Into<String>) -> Self {
        Self {
            value,
            currency: currency.into(),
        }
    }

    /// The raw magnitude.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The currency code.
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Add two amounts, requiring the same currency.
    pub fn checked_add(self, other: Amount) -> Result<Amount, EarnedValueError> {
        if self.currency != other.currency {
            return Err(EarnedValueError::CurrencyMismatch);
        }
        Ok(Amount::new(self.value + other.value, self.currency))
    }

    /// Subtract two amounts, requiring the same currency.
    pub fn checked_sub(self, other: Amount) -> Result<Amount, EarnedValueError> {
        if self.currency != other.currency {
            return Err(EarnedValueError::CurrencyMismatch);
        }
        Ok(Amount::new(self.value - other.value, self.currency))
    }
}

impl std::ops::Add for Amount {
    type Output = Amount;
    fn add(self, rhs: Amount) -> Amount {
        self.checked_add(rhs)
            .expect("Amount::Add requires matching currencies")
    }
}

impl std::ops::Sub for Amount {
    type Output = Amount;
    fn sub(self, rhs: Amount) -> Amount {
        self.checked_sub(rhs)
            .expect("Amount::Sub requires matching currencies")
    }
}

/// Strategy for forecasting Estimate at Completion (EAC).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EacMethod {
    /// Past performance is typical: `EAC = BAC / CPI`.
    Typical,
    /// Past performance is atypical (future work as planned): `EAC = AC + (BAC - EV)`.
    Atypical,
    /// Combined schedule + cost performance: `EAC = AC + (BAC - EV) / (CPI * SPI)`.
    Combined,
}

/// A single earned-value status snapshot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EarnedValueStatus {
    /// The project this status belongs to.
    pub project: ProjectId,
    /// Budget at Completion — the total authorized budget.
    pub bac: Amount,
    /// Planned Value (BCWS / PV): the budgeted cost of work scheduled.
    pub bcws: Amount,
    /// Earned Value (BCWP / EV): the budgeted cost of work performed.
    pub bcwp: Amount,
    /// Actual Cost (ACWP / AC): the actual cost of work performed.
    pub acwp: Amount,
}

impl EarnedValueStatus {
    /// Build a status snapshot. All monetary values must share a currency.
    pub fn new(
        project: ProjectId,
        bac: Amount,
        bcws: Amount,
        bcwp: Amount,
        acwp: Amount,
    ) -> Result<Self, EarnedValueError> {
        let cur = bac.currency().to_string();
        if bcws.currency() != cur || bcwp.currency() != cur || acwp.currency() != cur {
            return Err(EarnedValueError::CurrencyMismatch);
        }
        Ok(Self {
            project,
            bac,
            bcws,
            bcwp,
            acwp,
        })
    }

    /// Schedule Variance: `SV = EV - PV` (positive is ahead of plan).
    pub fn schedule_variance(&self) -> Amount {
        self.bcwp
            .clone()
            .checked_sub(self.bcws.clone())
            .expect("same currency")
    }

    /// Cost Variance: `CV = EV - AC` (positive is under budget).
    pub fn cost_variance(&self) -> Amount {
        self.bcwp
            .clone()
            .checked_sub(self.acwp.clone())
            .expect("same currency")
    }

    /// Schedule Performance Index: `SPI = EV / PV`. Returns `0.0` when `PV = 0`.
    pub fn spi(&self) -> f64 {
        ratio(self.bcwp.value(), self.bcws.value())
    }

    /// Cost Performance Index: `CPI = EV / AC`. Returns `0.0` when `AC = 0`.
    pub fn cpi(&self) -> f64 {
        ratio(self.bcwp.value(), self.acwp.value())
    }

    /// Percent complete by earned value: `EV / BAC`.
    pub fn percent_complete(&self) -> f64 {
        ratio(self.bcwp.value(), self.bac.value())
    }

    /// Estimate at Completion under the chosen forecasting method.
    pub fn eac(&self, method: EacMethod) -> Amount {
        let cur = self.bac.currency().to_string();
        let bac = self.bac.value();
        let ev = self.bcwp.value();
        let ac = self.acwp.value();
        let eac = match method {
            EacMethod::Typical => ratio(bac, self.cpi()),
            EacMethod::Atypical => ac + (bac - ev),
            EacMethod::Combined => {
                let denom = self.cpi() * self.spi();
                ac + ratio(bac - ev, denom)
            }
        };
        Amount::new(eac, cur)
    }

    /// Estimate to Complete: `EAC - AC`.
    pub fn etc(&self, method: EacMethod) -> Amount {
        self.eac(method)
            .checked_sub(self.acwp.clone())
            .expect("same currency")
    }

    /// Variance at Completion: `BAC - EAC`.
    pub fn vac(&self, method: EacMethod) -> Amount {
        self.bac
            .clone()
            .checked_sub(self.eac(method))
            .expect("same currency")
    }

    /// To-Complete Performance Index to finish at the BAC:
    /// `TCPI = (BAC - EV) / (BAC - AC)`. Returns `0.0` when the denominator is 0.
    pub fn tcpib(&self) -> f64 {
        ratio(
            self.bac.value() - self.bcwp.value(),
            self.bac.value() - self.acwp.value(),
        )
    }

    /// To-Complete Performance Index to finish at a given EAC:
    /// `TCPI = (BAC - EV) / (EAC - AC)`. Returns `0.0` when the denominator is 0.
    pub fn tcpie(&self, method: EacMethod) -> f64 {
        let eac = self.eac(method).value();
        ratio(
            self.bac.value() - self.bcwp.value(),
            eac - self.acwp.value(),
        )
    }
}

/// Errors raised while constructing earned-value status.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EarnedValueError {
    /// Monetary inputs used inconsistent currencies.
    #[error("earned value inputs must share a single currency")]
    CurrencyMismatch,
}

/// Divide `num / den`, returning `0.0` for a zero denominator (undefined index).
fn ratio(num: f64, den: f64) -> f64 {
    if den.abs() < f64::EPSILON {
        0.0
    } else {
        num / den
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::ProjectId;

    fn status() -> EarnedValueStatus {
        EarnedValueStatus::new(
            ProjectId::nil(),
            Amount::new(1000.0, "USD"),
            Amount::new(600.0, "USD"),
            Amount::new(500.0, "USD"),
            Amount::new(550.0, "USD"),
        )
        .unwrap()
    }

    #[test]
    fn variances() {
        let s = status();
        assert!((s.cost_variance().value() - (-50.0)).abs() < 1e-9);
        assert!((s.schedule_variance().value() - (-100.0)).abs() < 1e-9);
    }

    #[test]
    fn indices() {
        let s = status();
        assert!((s.spi() - 500.0 / 600.0).abs() < 1e-9);
        assert!((s.cpi() - 500.0 / 550.0).abs() < 1e-9);
        assert!((s.percent_complete() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn forecasts() {
        let s = status();
        let typical = s.eac(EacMethod::Typical);
        assert!((typical.value() - 1000.0 / (500.0 / 550.0)).abs() < 1e-6);
        let atypical = s.eac(EacMethod::Atypical);
        assert!((atypical.value() - (550.0 + 500.0)).abs() < 1e-9);
        let etc = s.etc(EacMethod::Atypical);
        assert!((etc.value() - 500.0).abs() < 1e-9);
        let vac = s.vac(EacMethod::Atypical);
        assert!((vac.value() - (-50.0)).abs() < 1e-9);
    }

    #[test]
    fn tcpis() {
        let s = status();
        assert!((s.tcpib() - 500.0 / 450.0).abs() < 1e-9);
        assert!((s.tcpie(EacMethod::Atypical) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_mismatched_currency() {
        let r = EarnedValueStatus::new(
            ProjectId::nil(),
            Amount::new(1000.0, "USD"),
            Amount::new(600.0, "USD"),
            Amount::new(500.0, "EUR"),
            Amount::new(550.0, "USD"),
        );
        assert_eq!(r, Err(EarnedValueError::CurrencyMismatch));
    }
}
