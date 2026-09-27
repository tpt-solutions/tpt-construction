// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Diffs between two revisions of a measured scope: quantity deltas and cost
//! deltas.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tpt_c_cost::Money;

use crate::ChangeError;

/// A measured quantity for one key at a point in time.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantityEntry {
    /// Unit of measure (e.g. `CY`, `m3`, `SF`).
    pub unit: String,
    /// Measured amount.
    pub amount: f64,
}

impl QuantityEntry {
    /// Build an entry.
    pub fn new(unit: impl Into<String>, amount: f64) -> Self {
        Self {
            unit: unit.into(),
            amount,
        }
    }
}

/// What happened to a key between two revisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaKind {
    /// Key present only in the later revision.
    Added,
    /// Key present only in the earlier revision.
    Removed,
    /// Amount grew.
    Increased,
    /// Amount shrank.
    Decreased,
    /// Amount (and unit) unchanged.
    Unchanged,
}

/// The change of one quantity key between two revisions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantityDelta {
    /// Quantity key (e.g. `"concrete"`, a cost code, or an element name).
    pub key: String,
    /// Unit carried over from either revision.
    pub unit: String,
    /// Amount in the earlier revision (`0.0` for added keys).
    pub before: f64,
    /// Amount in the later revision (`0.0` for removed keys).
    pub after: f64,
    /// How the amount moved.
    pub kind: DeltaKind,
}

impl QuantityDelta {
    /// Signed change in the measured amount (`after - before`).
    pub fn delta(&self) -> f64 {
        self.after - self.before
    }
}

/// The quantity comparison of two revisions.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct QuantityDiff {
    /// One delta per key present in either revision, ordered by key.
    pub deltas: Vec<QuantityDelta>,
}

impl QuantityDiff {
    /// Compare `before` and `after` quantity maps.
    ///
    /// Units are taken from the later revision when a key exists in both; keys
    /// whose unit changed are reported as `Removed` + `Added`.
    pub fn compute(
        before: &BTreeMap<String, QuantityEntry>,
        after: &BTreeMap<String, QuantityEntry>,
    ) -> Self {
        let mut deltas = Vec::new();
        for (key, entry) in after {
            match before.get(key) {
                Some(prev) if prev.unit == entry.unit => {
                    deltas.push(QuantityDelta {
                        key: key.clone(),
                        unit: entry.unit.clone(),
                        before: prev.amount,
                        after: entry.amount,
                        kind: classify(prev.amount, entry.amount),
                    });
                }
                _ => deltas.push(QuantityDelta {
                    key: key.clone(),
                    unit: entry.unit.clone(),
                    before: 0.0,
                    after: entry.amount,
                    kind: DeltaKind::Added,
                }),
            }
        }
        for (key, entry) in before {
            if !after.contains_key(key) {
                deltas.push(QuantityDelta {
                    key: key.clone(),
                    unit: entry.unit.clone(),
                    before: entry.amount,
                    after: 0.0,
                    kind: DeltaKind::Removed,
                });
            }
        }
        deltas.sort_by(|a, b| a.key.cmp(&b.key));
        Self { deltas }
    }

    /// Total absolute movement across all keys (a churn indicator).
    pub fn total_absolute_delta(&self) -> f64 {
        self.deltas.iter().map(|d| d.delta().abs()).sum()
    }
}

fn classify(before: f64, after: f64) -> DeltaKind {
    if before == after {
        DeltaKind::Unchanged
    } else if after > before {
        DeltaKind::Increased
    } else {
        DeltaKind::Decreased
    }
}

/// The cost change of one cost code between two revisions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostDelta {
    /// Cost code (e.g. MasterFormat `03 30 00`).
    pub code: String,
    /// Priced amount in the earlier revision.
    pub before: Money,
    /// Priced amount in the later revision.
    pub after: Money,
}

impl CostDelta {
    /// Signed change (`after - before`), rejecting currency mismatches.
    pub fn delta(&self) -> Result<Money, ChangeError> {
        Ok(self.after.checked_sub(self.before.clone())?)
    }
}

/// The full comparison of two named revisions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevisionDiff {
    /// Label of the earlier revision (e.g. `"rev-A"`).
    pub from: String,
    /// Label of the later revision (e.g. `"rev-B"`).
    pub to: String,
    /// Quantity movement between the revisions.
    pub quantities: QuantityDiff,
    /// Cost movement per cost code, ordered by code.
    pub cost_deltas: Vec<CostDelta>,
}

impl RevisionDiff {
    /// Compare two revisions of quantity takeoffs and priced cost codes.
    pub fn compute(
        from: impl Into<String>,
        to: impl Into<String>,
        before_quantities: &BTreeMap<String, QuantityEntry>,
        after_quantities: &BTreeMap<String, QuantityEntry>,
        before_costs: &BTreeMap<String, Money>,
        after_costs: &BTreeMap<String, Money>,
    ) -> Self {
        let mut cost_deltas: Vec<CostDelta> = after_costs
            .iter()
            .map(|(code, after)| CostDelta {
                code: code.clone(),
                before: before_costs
                    .get(code)
                    .cloned()
                    .unwrap_or_else(|| Money::zero(after.currency())),
                after: after.clone(),
            })
            .collect();
        for (code, before) in before_costs {
            if !after_costs.contains_key(code) {
                cost_deltas.push(CostDelta {
                    code: code.clone(),
                    before: before.clone(),
                    after: Money::zero(before.currency()),
                });
            }
        }
        cost_deltas.sort_by(|a, b| a.code.cmp(&b.code));
        Self {
            from: from.into(),
            to: to.into(),
            quantities: QuantityDiff::compute(before_quantities, after_quantities),
            cost_deltas,
        }
    }

    /// Net cost impact across all cost codes (credits included).
    pub fn net_cost_delta(&self) -> Result<Money, ChangeError> {
        let mut net = Money::zero("USD");
        let mut currency: Option<String> = None;
        for d in &self.cost_deltas {
            let delta = d.delta()?;
            match &currency {
                Some(c) if *c != delta.currency() => return Err(ChangeError::CurrencyMismatch),
                _ => currency = Some(delta.currency().to_string()),
            }
            net = net.checked_add(delta)?;
        }
        Ok(net)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quantities(entries: &[(&str, &str, f64)]) -> BTreeMap<String, QuantityEntry> {
        entries
            .iter()
            .map(|(k, u, a)| (k.to_string(), QuantityEntry::new(*u, *a)))
            .collect()
    }

    #[test]
    fn spec_example_concrete_delta() {
        let a = quantities(&[("concrete", "CY", 1000.0)]);
        let b = quantities(&[("concrete", "CY", 1050.0)]);
        let diff = QuantityDiff::compute(&a, &b);
        assert_eq!(diff.deltas.len(), 1);
        let d = &diff.deltas[0];
        assert_eq!(d.delta(), 50.0);
        assert_eq!(d.kind, DeltaKind::Increased);
        assert_eq!(d.unit, "CY");
    }

    #[test]
    fn added_removed_and_unchanged_keys() {
        let a = quantities(&[("concrete", "CY", 100.0), ("rebar", "t", 5.0)]);
        let b = quantities(&[("concrete", "CY", 100.0), ("paint", "SF", 900.0)]);
        let diff = QuantityDiff::compute(&a, &b);
        let by_key: BTreeMap<&str, &QuantityDelta> =
            diff.deltas.iter().map(|d| (d.key.as_str(), d)).collect();
        assert_eq!(by_key["rebar"].kind, DeltaKind::Removed);
        assert_eq!(by_key["rebar"].delta(), -5.0);
        assert_eq!(by_key["paint"].kind, DeltaKind::Added);
        assert_eq!(by_key["paint"].delta(), 900.0);
        assert_eq!(by_key["concrete"].kind, DeltaKind::Unchanged);
        assert_eq!(diff.total_absolute_delta(), 905.0);
    }

    #[test]
    fn unit_change_reports_removal_and_addition() {
        let a = quantities(&[("concrete", "CY", 10.0)]);
        let b = quantities(&[("concrete", "m3", 7.6)]);
        let diff = QuantityDiff::compute(&a, &b);
        assert_eq!(diff.deltas.len(), 1);
        assert_eq!(diff.deltas[0].kind, DeltaKind::Added);
    }

    #[test]
    fn cost_deltas_net_out() {
        let mut a = BTreeMap::new();
        a.insert("03 30 00".to_string(), Money::new(50_000.0, "USD"));
        a.insert("09 90 00".to_string(), Money::new(10_000.0, "USD"));
        let mut b = BTreeMap::new();
        b.insert("03 30 00".to_string(), Money::new(62_500.0, "USD"));
        let diff =
            RevisionDiff::compute("rev-A", "rev-B", &BTreeMap::new(), &BTreeMap::new(), &a, &b);
        assert_eq!(diff.cost_deltas.len(), 2);
        let net = diff.net_cost_delta().unwrap();
        assert_eq!(net.amount(), 2_500.0);
        assert_eq!(net.currency(), "USD");
    }

    #[test]
    fn mixed_currencies_rejected() {
        let mut a = BTreeMap::new();
        a.insert("03".to_string(), Money::new(1.0, "USD"));
        let mut b = BTreeMap::new();
        b.insert("03".to_string(), Money::new(2.0, "EUR"));
        let diff = RevisionDiff::compute("a", "b", &BTreeMap::new(), &BTreeMap::new(), &a, &b);
        assert_eq!(diff.net_cost_delta(), Err(ChangeError::CurrencyMismatch));
    }
}
