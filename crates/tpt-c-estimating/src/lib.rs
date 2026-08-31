// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construction estimating: turning a takeoff into a priced estimate.
//!
//! The central type is [`EstimateBuilder`], which consumes a [`TakeoffResult`]
//! (from `tpt-c-quantities`) and a [`CostDatabase`] (from `tpt-c-cost`), prices
//! each measured quantity by its cost code, and assembles an [`Estimate`]. This
//! crate also provides bid preparation, estimate revisions, comparison, and a
//! simple cost-plan rollup by code.

mod builder;
mod revision;

pub use builder::{EstimateBuilder, PricingError};
pub use revision::{EstimateComparison, EstimateRevision};

use serde::{Deserialize, Serialize};
use tpt_c_classification::Classification;
use tpt_c_core::AuditMeta;
use tpt_c_cost::{CostCode, Estimate, Markup};

/// A bid-prep wrapper around an estimate.
///
/// Carries the proposal metadata an estimator submits: a bid name, the estimate
/// itself, and an audit trail of who prepared it and when.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BidPreparation {
    /// Bid / proposal title.
    pub title: String,
    /// The underlying estimate.
    pub estimate: Estimate,
    /// Who prepared the bid and when.
    pub prepared_by: AuditMeta,
    /// Free-form notes.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

impl BidPreparation {
    /// Wrap an estimate in a bid package.
    pub fn new(title: impl Into<String>, estimate: Estimate, prepared_by: AuditMeta) -> Self {
        Self {
            title: title.into(),
            estimate,
            prepared_by,
            notes: String::new(),
        }
    }

    /// Total bid amount (estimate total including markup).
    pub fn total(&self) -> tpt_c_cost::Money {
        self.estimate
            .total()
            .unwrap_or_else(|_| tpt_c_cost::Money::zero(&self.estimate.currency))
    }
}

/// A cost plan groups priced quantities by cost code, giving a by-division
/// budget rollup (e.g. per MasterFormat division).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CostPlan {
    /// Rollup rows keyed by cost code.
    pub rows: std::collections::BTreeMap<String, CostPlanRow>,
}

/// One cost-plan rollup row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostPlanRow {
    /// Cost code.
    pub code: String,
    /// Title when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Number of line items contributing.
    pub line_count: usize,
    /// Total extended cost for this code.
    pub total: tpt_c_cost::Money,
}

impl CostPlan {
    /// Build a cost plan from an estimate's line items.
    pub fn from_estimate(estimate: &Estimate) -> Self {
        let mut rows = std::collections::BTreeMap::new();
        for line in &estimate.line_items {
            let entry = rows
                .entry(line.code.code.clone())
                .or_insert_with(|| CostPlanRow {
                    code: line.code.code.clone(),
                    title: line.code.title.clone(),
                    line_count: 0,
                    total: tpt_c_cost::Money::zero(&estimate.currency),
                });
            entry.line_count += 1;
            entry.total = entry
                .total
                .checked_add(line.extension.clone())
                .unwrap_or_else(|_| entry.total.clone());
        }
        Self { rows }
    }

    /// The grand total across all codes.
    pub fn total(&self) -> tpt_c_cost::Money {
        self.rows
            .values()
            .fold(tpt_c_cost::Money::zero("USD"), |acc, r| {
                acc.checked_add(r.total.clone()).unwrap_or(acc)
            })
    }
}

/// Convenience: derive a cost code from an element's classification, falling
/// back to its category.
pub fn code_for(classification: &Option<Classification>, category: &str) -> CostCode {
    match classification {
        Some(c) => CostCode::from_classification(c),
        None => CostCode::new(category),
    }
}

/// A default markup schedule suitable for a design-stage estimate.
pub fn default_markup() -> Markup {
    Markup::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_classification::{Classification, ClassificationSystem};
    use tpt_c_core::ProjectId;
    use tpt_c_cost::{CostDatabase, Money, RateUnit, ResourceKind, ResourceRate};
    use tpt_c_ids::IdFactory;
    use tpt_c_model::{Element, Project, Quantity, QuantitySet};
    use tpt_c_quantities::{TakeoffEngine, TakeoffResult};
    use tpt_c_units::Volume;

    fn sample_project() -> Project {
        let mut p = Project::new(ProjectId::nil(), "Demo");
        let e = Element::new(IdFactory::element(), "S1", "Slab")
            .classified(Classification::new(
                ClassificationSystem::MasterFormat,
                "03 30 00",
            ))
            .with_quantity_set(QuantitySet::new("Q").with(
                "GrossVolume",
                Quantity::Volume(Volume::from_cubic_yards(10.0)),
            ));
        p.add_element(e);
        p
    }

    fn sample_db() -> CostDatabase {
        let mut db = CostDatabase::new();
        db.insert(
            "03 30 00",
            ResourceRate::new(
                "r1",
                ResourceKind::Material,
                RateUnit::Volume,
                Money::new(120.0, "USD"),
            )
            .with_description("Concrete CY"),
        );
        db
    }

    #[test]
    fn build_estimate_from_takeoff() {
        let project = sample_project();
        let takeoff: TakeoffResult = TakeoffEngine::new().run(&project);
        let builder = EstimateBuilder::new("Demo Estimate", "USD").with_database(sample_db());
        let estimate = builder.from_takeoff(&takeoff).unwrap();
        assert_eq!(estimate.line_items.len(), 1);
        let totals = estimate.totals().unwrap();
        // 10 CY gross-priced at 120 => 917.46 subtotal, plus markup > subtotal
        assert!(totals.subtotal.amount() > 900.0);
        assert!(totals.total.amount() > totals.subtotal.amount());
    }

    #[test]
    fn cost_plan_rollup() {
        let project = sample_project();
        let takeoff = TakeoffEngine::new().run(&project);
        let estimate = EstimateBuilder::new("E", "USD")
            .with_database(sample_db())
            .from_takeoff(&takeoff)
            .unwrap();
        let plan = CostPlan::from_estimate(&estimate);
        assert_eq!(plan.rows.len(), 1);
        assert!(plan.total().amount() > 0.0);
    }

    #[test]
    fn revision_compare() {
        let project = sample_project();
        let takeoff = TakeoffEngine::new().run(&project);
        let a = EstimateBuilder::new("E", "USD")
            .with_database(sample_db())
            .from_takeoff(&takeoff)
            .unwrap();
        let b = a.clone();
        let rev_a = EstimateRevision::new(a, "baseline");
        let rev_b = EstimateRevision::new(b, "no-change");
        let cmp = EstimateRevision::compare(&rev_a, &rev_b);
        assert!((cmp.total_delta.amount()).abs() < 1e-9);
    }
}
