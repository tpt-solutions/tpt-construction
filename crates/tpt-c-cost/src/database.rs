// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cost codes and the resource-rate database used to price quantities by code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tpt_c_classification::Classification;

use crate::resource::{RateUnit, ResourceKind, ResourceRate};
use crate::Money;

/// A cost code: a classification-style key used to group and price work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CostCode {
    /// The code (e.g. MasterFormat `03 30 00`).
    pub code: String,
    /// Human-readable title when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl CostCode {
    /// Build a cost code.
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            title: None,
        }
    }

    /// Attach a title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Derive a cost code from a classification.
    pub fn from_classification(c: &Classification) -> Self {
        Self {
            code: c.code.clone(),
            title: c.title.clone(),
        }
    }
}

/// A lookup table mapping cost codes to resource rates.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CostDatabase {
    rates: HashMap<String, ResourceRate>,
}

impl CostDatabase {
    /// An empty database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a rate under its cost code.
    pub fn insert(&mut self, code: impl Into<String>, rate: ResourceRate) {
        self.rates.insert(code.into(), rate);
    }

    /// Look up a rate by cost code.
    pub fn get(&self, code: &str) -> Option<&ResourceRate> {
        self.rates.get(code)
    }

    /// All registered rates.
    pub fn rates(&self) -> impl Iterator<Item = &ResourceRate> {
        self.rates.values()
    }

    /// The number of registered rates.
    pub fn len(&self) -> usize {
        self.rates.len()
    }

    /// Whether the database is empty.
    pub fn is_empty(&self) -> bool {
        self.rates.is_empty()
    }

    /// Convenience: build a [`ResourceRate`] from parts and insert it.
    pub fn add_rate(
        &mut self,
        code: impl Into<String>,
        id: impl Into<String>,
        kind: ResourceKind,
        unit: RateUnit,
        rate: Money,
    ) {
        self.insert(code.into(), ResourceRate::new(id, kind, unit, rate));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_classification::ClassificationSystem;

    #[test]
    fn code_from_classification() {
        let c = Classification::new(ClassificationSystem::MasterFormat, "03 30 00")
            .with_title("Cast-in-Place Concrete");
        let code = CostCode::from_classification(&c);
        assert_eq!(code.code, "03 30 00");
        assert_eq!(code.title.as_deref(), Some("Cast-in-Place Concrete"));
    }

    #[test]
    fn database_lookup() {
        let mut db = CostDatabase::new();
        db.add_rate(
            "03 30 00",
            "r1",
            ResourceKind::Material,
            RateUnit::Volume,
            Money::new(120.0, "USD"),
        );
        assert_eq!(db.get("03 30 00").unwrap().rate.amount(), 120.0);
        assert!(db.get("09 00 00").is_none());
    }
}
