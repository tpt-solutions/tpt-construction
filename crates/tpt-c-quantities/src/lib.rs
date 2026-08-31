// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Quantity takeoff engine for construction estimating.
//!
//! A *takeoff* converts a neutral [`Project`](tpt_c_model::Project) model into a
//! set of measurable quantities (count, length, area, volume, weight). Each
//! quantity is tracked as a **net** value (taken from the model) and a **gross**
//! value (net expanded by a waste factor). Quantities imported from the model
//! can be locally overridden by an estimator.
//!
//! The crate also ships a derived *rule set* (`rules`) that computes quantities
//! a parser may not have extracted directly — concrete volume, formwork area,
//! rebar weight, paint area, and flooring area with deductions.

mod engine;
mod rules;

pub use engine::{TakeoffEngine, TakeoffItem, TakeoffResult};
pub use rules::{QuantityRules, DEFAULT_WASTE_FORMWORK, DEFAULT_WASTE_PAINT, DEFAULT_WASTE_REBAR};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tpt_c_core::ElementId;
use tpt_c_units::{Area, Count, Length, Mass, Volume, WasteFactor, Wasteable};

/// The kind of a measured quantity. Used as a stable key when assigning waste
/// factors and cost codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityKind {
    /// Discrete count ("each").
    Count,
    /// Linear measure (LF).
    Length,
    /// Area measure (SF).
    Area,
    /// Volume measure (CY).
    Volume,
    /// Mass (ton).
    Mass,
}

/// A quantity in its canonical base unit (the same SI base units used by
/// [`tpt_c_units`]).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum MeasuredQuantity {
    /// Discrete count.
    Count(Count),
    /// Linear measure.
    Length(Length),
    /// Area measure.
    Area(Area),
    /// Volume measure.
    Volume(Volume),
    /// Mass.
    Mass(Mass),
}

impl MeasuredQuantity {
    /// The quantity kind.
    pub fn kind(&self) -> QuantityKind {
        match self {
            MeasuredQuantity::Count(_) => QuantityKind::Count,
            MeasuredQuantity::Length(_) => QuantityKind::Length,
            MeasuredQuantity::Area(_) => QuantityKind::Area,
            MeasuredQuantity::Volume(_) => QuantityKind::Volume,
            MeasuredQuantity::Mass(_) => QuantityKind::Mass,
        }
    }

    /// The magnitude in the canonical base unit.
    pub fn base_value(&self) -> f64 {
        match self {
            MeasuredQuantity::Count(c) => c.each(),
            MeasuredQuantity::Length(l) => l.meters(),
            MeasuredQuantity::Area(a) => a.square_meters(),
            MeasuredQuantity::Volume(v) => v.cubic_meters(),
            MeasuredQuantity::Mass(m) => m.kilograms(),
        }
    }

    /// Expand the quantity by a waste factor, returning the gross amount.
    pub fn apply_waste(self, waste: WasteFactor) -> Self {
        match self {
            MeasuredQuantity::Count(c) => MeasuredQuantity::Count(c.apply_waste(waste)),
            MeasuredQuantity::Length(l) => MeasuredQuantity::Length(l.apply_waste(waste)),
            MeasuredQuantity::Area(a) => MeasuredQuantity::Area(a.apply_waste(waste)),
            MeasuredQuantity::Volume(v) => MeasuredQuantity::Volume(v.apply_waste(waste)),
            MeasuredQuantity::Mass(m) => MeasuredQuantity::Mass(m.apply_waste(waste)),
        }
    }

    /// Build from a [`tpt_c_model::Quantity`].
    pub fn from_model(q: &tpt_c_model::Quantity) -> Option<Self> {
        match q {
            tpt_c_model::Quantity::Count(c) => Some(MeasuredQuantity::Count(*c)),
            tpt_c_model::Quantity::Length(l) => Some(MeasuredQuantity::Length(*l)),
            tpt_c_model::Quantity::Area(a) => Some(MeasuredQuantity::Area(*a)),
            tpt_c_model::Quantity::Volume(v) => Some(MeasuredQuantity::Volume(*v)),
            tpt_c_model::Quantity::Mass(m) => Some(MeasuredQuantity::Mass(*m)),
            tpt_c_model::Quantity::Duration(_) => None,
        }
    }
}

impl From<MeasuredQuantity> for tpt_c_model::Quantity {
    fn from(m: MeasuredQuantity) -> Self {
        match m {
            MeasuredQuantity::Count(c) => tpt_c_model::Quantity::Count(c),
            MeasuredQuantity::Length(l) => tpt_c_model::Quantity::Length(l),
            MeasuredQuantity::Area(a) => tpt_c_model::Quantity::Area(a),
            MeasuredQuantity::Volume(v) => tpt_c_model::Quantity::Volume(v),
            MeasuredQuantity::Mass(m) => tpt_c_model::Quantity::Mass(m),
        }
    }
}

/// A single named, measurable quantity on a takeoff item.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TakeoffQuantity {
    /// Quantity name (e.g. `GrossVolume`).
    pub name: &'static str,
    /// Net value taken from the model (or a manual override).
    pub net: MeasuredQuantity,
    /// Waste factor applied to produce the gross amount.
    pub waste: WasteFactor,
    /// `true` when `net` came from an estimator override rather than the model.
    pub manual: bool,
}

impl TakeoffQuantity {
    /// Gross value = net expanded by the waste factor.
    pub fn gross(&self) -> MeasuredQuantity {
        self.net.apply_waste(self.waste)
    }
}

/// A quantity lookup failure during takeoff.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TakeoffError {
    /// No quantity of the requested kind was found on an element.
    #[error("no {0:?} quantity available for element {1}")]
    MissingQuantity(QuantityKind, ElementId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_units::WasteFactor;

    #[test]
    fn gross_expands_by_waste() {
        let q = TakeoffQuantity {
            name: "GrossVolume",
            net: MeasuredQuantity::Volume(Volume::from_cubic_yards(100.0)),
            waste: WasteFactor::from_percent(8.0),
            manual: false,
        };
        assert!((q.gross().base_value() / q.net.base_value() - 1.08).abs() < 1e-9);
    }

    #[test]
    fn from_model_roundtrip() {
        let m = MeasuredQuantity::from_model(&tpt_c_model::Quantity::Area(Area::from_square_feet(
            50.0,
        )));
        assert_eq!(
            m,
            Some(MeasuredQuantity::Area(Area::from_square_feet(50.0)))
        );
    }
}
