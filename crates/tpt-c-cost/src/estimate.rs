// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Estimates, markups, and budgets.

use serde::{Deserialize, Serialize};
use tpt_c_core::{EstimateId, Identified};
use tpt_c_ids::IdFactory;

use crate::{CostError, LineItem, Money, round_money};

/// Markup percentages applied to an estimate subtotal.
///
/// Applied in order: overhead → profit → escalation → tax. Each percentage is
/// expressed in percent points (e.g. `10.0` = 10%).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Markup {
    /// General conditions / overhead, applied to the subtotal.
    pub overhead_pct: f64,
    /// Profit, applied to (subtotal + overhead).
    pub profit_pct: f64,
    /// Cost escalation, applied to (subtotal + overhead + profit).
    pub escalation_pct: f64,
    /// Tax, applied last to the post-escalation amount.
    pub tax_pct: f64,
}

impl Default for Markup {
    fn default() -> Self {
        Self {
            overhead_pct: 10.0,
            profit_pct: 8.0,
            escalation_pct: 0.0,
            tax_pct: 0.0,
        }
    }
}

impl Markup {
    /// A markup with all percentages zero.
    pub fn none() -> Self {
        Self {
            overhead_pct: 0.0,
            profit_pct: 0.0,
            escalation_pct: 0.0,
            tax_pct: 0.0,
        }
    }

    /// Set the overhead percentage.
    pub fn with_overhead(mut self, pct: f64) -> Self {
        self.overhead_pct = pct;
        self
    }

    /// Set the profit percentage.
    pub fn with_profit(mut self, pct: f64) -> Self {
        self.profit_pct = pct;
        self
    }

    /// Set the escalation percentage.
    pub fn with_escalation(mut self, pct: f64) -> Self {
        self.escalation_pct = pct;
        self
    }

    /// Set the tax percentage.
    pub fn with_tax(mut self, pct: f64) -> Self {
        self.tax_pct = pct;
        self
    }

    /// Compute the markup components for a subtotal.
    pub fn totals(&self, subtotal: Money) -> MarkupTotals {
        let cur = subtotal.currency().to_string();
        let s = subtotal.amount();
        let overhead = round_money(s * self.overhead_pct / 100.0);
        let wo = s + overhead;
        let profit = round_money(wo * self.profit_pct / 100.0);
        let wp = wo + profit;
        let escalation = round_money(wp * self.escalation_pct / 100.0);
        let we = wp + escalation;
        let tax = round_money(we * self.tax_pct / 100.0);
        let total = we + tax;
        let m = |v: f64| Money::new(round_money(v), cur.clone());
        MarkupTotals {
            subtotal: m(s),
            overhead: m(overhead),
            profit: m(profit),
            escalation: m(escalation),
            tax: m(tax),
            total: m(total),
        }
    }
}

/// The fully-computed monetary breakdown of an estimate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkupTotals {
    /// Net subtotal before markup.
    pub subtotal: Money,
    /// Overhead amount.
    pub overhead: Money,
    /// Profit amount.
    pub profit: Money,
    /// Escalation amount.
    pub escalation: Money,
    /// Tax amount.
    pub tax: Money,
    /// Final total.
    pub total: Money,
}

/// A priced estimate: an ordered set of line items plus a markup schedule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Estimate {
    /// Estimate identifier.
    pub id: EstimateId,
    /// Estimate title.
    pub title: String,
    /// Currency code for all amounts.
    pub currency: String,
    /// Line items in presentation order.
    pub line_items: Vec<LineItem>,
    /// Markup schedule.
    pub markup: Markup,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

impl Estimate {
    /// Build an empty estimate.
    pub fn new(title: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            id: IdFactory::estimate(),
            title: title.into(),
            currency: currency.into(),
            line_items: Vec::new(),
            markup: Markup::default(),
            notes: String::new(),
        }
    }

    /// Append a line item, assigning the next sequence number.
    pub fn add_line(&mut self, line: LineItem) {
        self.line_items.push(line);
    }

    /// Replace the markup schedule.
    pub fn with_markup(mut self, markup: Markup) -> Self {
        self.markup = markup;
        self
    }

    /// Attach a note.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    /// Sum of line-item extensions.
    pub fn subtotal(&self) -> Result<Money, CostError> {
        self.line_items
            .iter()
            .try_fold(Money::zero(&self.currency), |acc, l| acc.checked_add(l.extension))
    }

    /// Subtotal with the full markup schedule applied.
    pub fn totals(&self) -> Result<MarkupTotals, CostError> {
        let subtotal = self.subtotal()?;
        Ok(self.markup.totals(subtotal))
    }

    /// Total bid amount (subtotal + markup).
    pub fn total(&self) -> Result<Money, CostError> {
        Ok(self.totals()?.total)
    }
}

impl Identified for Estimate {
    type Id = EstimateId;
    fn id(&self) -> EstimateId {
        self.id
    }
}

/// A project budget: an allowed amount plus a contingency reserve.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    /// Currency code.
    pub currency: String,
    /// The approved allowance (before contingency).
    pub allowance: Money,
    /// Contingency percentage.
    pub contingency_pct: f64,
    /// Contingency amount.
    pub contingency: Money,
    /// Allowance + contingency.
    pub total: Money,
}

impl Budget {
    /// Build a budget with a contingency percentage.
    pub fn new(allowance: Money, contingency_pct: f64) -> Self {
        let contingency = allowance.scaled(contingency_pct / 100.0).rounded(2);
        let total = (allowance + contingency).rounded(2);
        Self {
            currency: allowance.currency().to_string(),
            allowance: allowance.rounded(2),
            contingency_pct,
            contingency,
            total,
        }
    }

    /// Compare an estimate total against this budget, returning the delta
    /// (`budget.total - estimate_total`); positive means under budget.
    pub fn variance(&self, estimate_total: Money) -> Result<Money, CostError> {
        self.total.checked_sub(estimate_total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_units::Volume;

    #[test]
    fn markup_order() {
        let m = Markup::none().with_overhead(10.0).with_profit(10.0);
        let t = m.totals(Money::new(100.0, "USD"));
        // 100 + 10 overhead = 110; profit 10% of 110 = 11; total 121
        assert_eq!(t.overhead.amount(), 10.0);
        assert_eq!(t.profit.amount(), 11.0);
        assert_eq!(t.total.amount(), 121.0);
    }

    #[test]
    fn estimate_totals() {
        let mut e = Estimate::new("Demo", "USD");
        e.add_line(LineItem::new(
            1,
            crate::CostCode::new("03 30 00"),
            tpt_c_model::Quantity::Volume(Volume::from_cubic_yards(10.0)),
            Money::new(120.0, "USD"),
        ));
        let totals = e.totals().unwrap();
        assert!((totals.subtotal.amount() - 917.46).abs() < 0.01);
        assert!(totals.total.amount() > totals.subtotal.amount());
    }

    #[test]
    fn budget_variance() {
        let b = Budget::new(Money::new(1000.0, "USD"), 10.0);
        assert_eq!(b.contingency.amount(), 100.0);
        let v = b.variance(Money::new(950.0, "USD")).unwrap();
        assert_eq!(v.amount(), 150.0);
    }
}
