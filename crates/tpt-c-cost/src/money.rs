// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Monetary amounts with currency-aware arithmetic.

use serde::{Deserialize, Serialize};
use tpt_c_units::round_to_decimals;

use crate::CostError;

/// A monetary amount in a specific ISO 4217 currency.
///
/// Arithmetic carries the currency; mismatched currencies are rejected rather
/// than silently combined.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Money {
    /// Magnitude in the major unit (e.g. dollars, not cents).
    amount: f64,
    /// ISO 4217 currency code (e.g. `USD`).
    currency: String,
}

impl Money {
    /// Build an amount in `currency`.
    pub fn new(amount: f64, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
        }
    }

    /// A zero amount in `currency`.
    pub fn zero(currency: impl Into<String>) -> Self {
        Self::new(0.0, currency)
    }

    /// The raw magnitude.
    pub fn amount(&self) -> f64 {
        self.amount
    }

    /// The currency code.
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Round to `decimals` places (default-style half-away-from-zero).
    pub fn rounded(self, decimals: u32) -> Self {
        Self::new(
            round_to_decimals(self.amount, decimals),
            self.currency.clone(),
        )
    }

    /// Add two amounts, requiring the same currency.
    pub fn checked_add(&self, other: Money) -> Result<Money, CostError> {
        if self.currency != other.currency {
            return Err(CostError::CurrencyMismatch {
                lhs: self.currency.clone(),
                rhs: other.currency,
            });
        }
        Ok(Money::new(
            self.amount + other.amount,
            self.currency.clone(),
        ))
    }

    /// Subtract two amounts, requiring the same currency.
    pub fn checked_sub(&self, other: Money) -> Result<Money, CostError> {
        if self.currency != other.currency {
            return Err(CostError::CurrencyMismatch {
                lhs: self.currency.clone(),
                rhs: other.currency,
            });
        }
        Ok(Money::new(
            self.amount - other.amount,
            self.currency.clone(),
        ))
    }

    /// Scale by a dimensionless factor (e.g. a quantity or a percentage).
    pub fn scaled(&self, factor: f64) -> Money {
        Money::new(self.amount * factor, self.currency.clone())
    }
}

impl std::ops::Add for Money {
    type Output = Money;
    fn add(self, rhs: Money) -> Money {
        self.checked_add(rhs)
            .expect("Money::Add requires matching currencies")
    }
}

impl std::ops::Sub for Money {
    type Output = Money;
    fn sub(self, rhs: Money) -> Money {
        self.checked_sub(rhs)
            .expect("Money::Sub requires matching currencies")
    }
}

impl std::ops::Mul<f64> for Money {
    type Output = Money;
    fn mul(self, rhs: f64) -> Money {
        self.scaled(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_currency_adds() {
        let a = Money::new(10.0, "USD");
        let b = Money::new(5.0, "USD");
        assert_eq!((a + b).amount(), 15.0);
    }

    #[test]
    fn mismatch_rejected() {
        let a = Money::new(10.0, "USD");
        let b = Money::new(5.0, "EUR");
        assert!(a.checked_add(b).is_err());
    }

    #[test]
    fn rounding() {
        assert_eq!(Money::new(2.345, "USD").rounded(2).amount(), 2.35);
    }
}
