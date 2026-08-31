// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Estimate revisions and comparison.

use serde::{Deserialize, Serialize};
use tpt_c_core::EstimateId;
use tpt_c_cost::{Estimate, Money};
use tpt_c_ids::IdFactory;

/// A versioned estimate with an audit note.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstimateRevision {
    /// Revision identifier.
    pub id: EstimateId,
    /// The estimate this revises, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<EstimateId>,
    /// Human-readable revision label (e.g. `v2-design-development`).
    pub label: String,
    /// The estimate at this revision.
    pub estimate: Estimate,
}

impl EstimateRevision {
    /// Create a revision of `estimate` labelled `label`.
    pub fn new(estimate: Estimate, label: impl Into<String>) -> Self {
        let parent = estimate.id;
        Self {
            id: IdFactory::estimate(),
            parent: Some(parent),
            label: label.into(),
            estimate,
        }
    }

    /// The total amount at this revision.
    pub fn total(&self) -> Money {
        self.estimate
            .total()
            .unwrap_or_else(|_| Money::zero(&self.estimate.currency))
    }

    /// Compare two revisions, returning the delta and per-code differences.
    pub fn compare(a: &EstimateRevision, b: &EstimateRevision) -> EstimateComparison {
        let total_delta = a
            .total()
            .checked_sub(b.total())
            .unwrap_or_else(|_| Money::zero(&a.estimate.currency));
        let sum_by_code = |e: &Estimate| -> std::collections::BTreeMap<String, Money> {
            let mut m = std::collections::BTreeMap::new();
            for line in &e.line_items {
                let cur = m
                    .entry(line.code.code.clone())
                    .or_insert_with(|| Money::zero(&e.currency));
                *cur = cur
                    .checked_add(line.extension.clone())
                    .unwrap_or_else(|_| cur.clone());
            }
            m
        };
        let a_codes = sum_by_code(&a.estimate);
        let b_codes = sum_by_code(&b.estimate);
        let mut by_code: std::collections::BTreeMap<String, Money> =
            std::collections::BTreeMap::new();
        for code in a_codes.keys().chain(b_codes.keys()) {
            let av = a_codes
                .get(code)
                .cloned()
                .unwrap_or_else(|| Money::zero(&a.estimate.currency));
            let bv = b_codes
                .get(code)
                .cloned()
                .unwrap_or_else(|| Money::zero(&a.estimate.currency));
            let delta = av
                .checked_sub(bv)
                .unwrap_or_else(|_| Money::zero(&a.estimate.currency));
            by_code.insert(code.clone(), delta);
        }
        EstimateComparison {
            from_label: a.label.clone(),
            to_label: b.label.clone(),
            total_delta,
            line_count_delta: a.estimate.line_items.len() as i64
                - b.estimate.line_items.len() as i64,
            by_code,
        }
    }
}

/// The computed difference between two estimate revisions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstimateComparison {
    /// Source revision label.
    pub from_label: String,
    /// Target revision label.
    pub to_label: String,
    /// Change in total (`from - to`); positive means `from` is higher.
    pub total_delta: Money,
    /// Change in line-item count.
    pub line_count_delta: i64,
    /// Per-code total deltas (`from - to`).
    pub by_code: std::collections::BTreeMap<String, Money>,
}
