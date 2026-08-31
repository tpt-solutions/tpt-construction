// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Priced cost items, assemblies, and estimate line items.

use serde::{Deserialize, Serialize};
use tpt_c_core::Identified;
use tpt_c_ids::IdFactory;
use tpt_c_model::Quantity;

use crate::{round_money, CostCode, CostError, Money};

/// A single priced quantity: a cost code, a quantity, and a unit rate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostItem {
    /// Stable item id.
    pub id: String,
    /// Cost code this item is grouped under.
    pub code: CostCode,
    /// Description.
    pub description: String,
    /// The quantity being priced.
    pub quantity: Quantity,
    /// Unit rate (per one base unit of `quantity`).
    pub unit_rate: Money,
    /// Extended total = `quantity.base_value() * unit_rate.amount`.
    pub total: Money,
}

impl CostItem {
    /// Build a cost item, computing its extended total.
    pub fn new(code: CostCode, quantity: Quantity, unit_rate: Money) -> Self {
        let description = code.title.clone().unwrap_or_else(|| code.code.clone());
        let total = Money::new(
            round_money(quantity.base_value() * unit_rate.amount()),
            unit_rate.currency(),
        );
        Self {
            id: IdFactory::uuid7().to_string(),
            code,
            description,
            quantity,
            unit_rate,
            total,
        }
    }

    /// Build with an explicit description.
    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = d.into();
        self
    }
}

impl Identified for CostItem {
    type Id = String;
    fn id(&self) -> String {
        self.id.clone()
    }
}

/// A cost assembly: a composite item made of several cost items.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostAssembly {
    /// Stable assembly id.
    pub id: String,
    /// Cost code for the assembly.
    pub code: CostCode,
    /// Description.
    pub description: String,
    /// Constituent cost items.
    pub items: Vec<CostItem>,
}

impl CostAssembly {
    /// Build an assembly.
    pub fn new(code: CostCode, items: Vec<CostItem>) -> Self {
        Self {
            id: IdFactory::uuid7().to_string(),
            description: code.title.clone().unwrap_or_else(|| code.code.clone()),
            code,
            items,
        }
    }

    /// The rolled-up total of all constituent items.
    pub fn total(&self) -> Result<Money, CostError> {
        self.items
            .iter()
            .try_fold(Money::zero(self.code_title_currency()), |acc, i| {
                acc.checked_add(i.total.clone())
            })
    }

    fn code_title_currency(&self) -> &str {
        self.items
            .first()
            .map(|i| i.total.currency())
            .unwrap_or("USD")
    }
}

/// A row on an estimate: a sequenced, priced line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineItem {
    /// Sequence number on the estimate.
    pub number: usize,
    /// Cost code.
    pub code: CostCode,
    /// Description.
    pub description: String,
    /// Quantity.
    pub quantity: Quantity,
    /// Unit rate.
    pub unit_rate: Money,
    /// Extended amount.
    pub extension: Money,
}

impl LineItem {
    /// Build a line item, computing its extension.
    pub fn new(number: usize, code: CostCode, quantity: Quantity, unit_rate: Money) -> Self {
        let description = code.title.clone().unwrap_or_else(|| code.code.clone());
        let extension = Money::new(
            round_money(quantity.base_value() * unit_rate.amount()),
            unit_rate.currency(),
        );
        Self {
            number,
            code,
            description,
            quantity,
            unit_rate,
            extension,
        }
    }

    /// Build from a [`CostItem`], assigning a sequence number.
    pub fn from_item(number: usize, item: &CostItem) -> Self {
        Self {
            number,
            code: item.code.clone(),
            description: item.description.clone(),
            quantity: item.quantity,
            unit_rate: item.unit_rate.clone(),
            extension: item.total.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_units::{Area, Volume};

    #[test]
    fn cost_item_total() {
        let item = CostItem::new(
            CostCode::new("03 30 00"),
            Quantity::Volume(Volume::from_cubic_yards(10.0)),
            Money::new(120.0, "USD"),
        );
        // 10 CY = 7.6455 m3 * 120 = 917.46
        assert!((item.total.amount() - 917.46).abs() < 0.01);
    }

    #[test]
    fn line_item_from_item() {
        let item = CostItem::new(
            CostCode::new("09 00 00"),
            Quantity::Area(Area::from_square_feet(100.0)),
            Money::new(3.0, "USD"),
        );
        let line = LineItem::from_item(1, &item);
        assert_eq!(line.number, 1);
        assert!((line.extension.amount() - 27.87).abs() < 0.01);
    }

    #[test]
    fn assembly_total() {
        let a = CostItem::new(
            CostCode::new("A"),
            Quantity::Count(tpt_c_units::Count::from_each(1.0)),
            Money::new(10.0, "USD"),
        );
        let b = CostItem::new(
            CostCode::new("B"),
            Quantity::Count(tpt_c_units::Count::from_each(2.0)),
            Money::new(5.0, "USD"),
        );
        let asm = CostAssembly::new(CostCode::new("ASM"), vec![a, b]);
        assert_eq!(asm.total().unwrap().amount(), 20.0);
    }
}
