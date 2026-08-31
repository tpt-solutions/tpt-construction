// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Schedule of values: the contract broken into valued line items.

use serde::{Deserialize, Serialize};
use tpt_c_cost::Money;

/// A single line item in a schedule of values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SovItem {
    /// Item identifier (often the cost code).
    pub id: String,
    /// Description.
    pub description: String,
    /// Contracted value.
    pub contract_value: Money,
}

/// A schedule of values.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduleOfValues {
    /// SOV identifier.
    pub id: String,
    /// Line items.
    #[serde(default)]
    pub items: Vec<SovItem>,
}

impl ScheduleOfValues {
    /// Create an empty SOV.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
        }
    }

    /// Add a line item, returning it.
    pub fn add_item(&mut self, id: impl Into<String>, description: impl Into<String>, contract_value: Money) -> SovItem {
        let item = SovItem {
            id: id.into(),
            description: description.into(),
            contract_value,
        };
        self.items.push(item.clone());
        item
    }

    /// Total contracted value across all items.
    pub fn total(&self) -> Money {
        let mut total = None;
        for item in &self.items {
            total = Some(match total {
                None => item.contract_value.clone(),
                Some(t) => t + item.contract_value.clone(),
            });
        }
        total.unwrap_or_else(|| Money::zero("USD"))
    }

    /// Number of line items.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the SOV has no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_total_is_zero() {
        let sov = ScheduleOfValues::new("SOV-0");
        assert_eq!(sov.total().amount(), 0.0);
        assert!(sov.is_empty());
    }
}
