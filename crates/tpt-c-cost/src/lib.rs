// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construction cost model.
//!
//! This crate defines the vocabulary for pricing a takeoff: resource rates
//! (labor / material / equipment / subcontractor), cost items and assemblies
//! that price a [`Quantity`](tpt_c_model::Quantity), line items on an estimate,
//! and the markup math (overhead, profit, tax, escalation) that turns a subtotal
//! into a bid total. A [`CostDatabase`] maps cost codes to resource rates so an
//! estimator can price quantities by code.

mod database;
mod estimate;
mod items;
mod money;
mod resource;

pub use database::{CostCode, CostDatabase};
pub use estimate::{Budget, Estimate, Markup, MarkupTotals};
pub use items::{CostAssembly, CostItem, LineItem};
pub use money::Money;
pub use resource::{RateUnit, ResourceKind, ResourceRate};

/// Errors raised while working with cost values.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CostError {
    /// Two monetary amounts had different currencies and could not be combined.
    #[error("currency mismatch: {lhs} vs {rhs}")]
    CurrencyMismatch {
        /// Left-hand currency.
        lhs: String,
        /// Right-hand currency.
        rhs: String,
    },

    /// A cost database had no rate for the requested code.
    #[error("no rate for cost code {0}")]
    MissingRate(String),

    /// A required quantity was absent.
    #[error("missing quantity: {0}")]
    MissingQuantity(String),
}

/// Round a monetary amount to cents (2 decimals).
pub fn round_money(amount: f64) -> f64 {
    tpt_c_units::round_to_decimals(amount, 2)
}
