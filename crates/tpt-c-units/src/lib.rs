// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construction measurement units and domain measure helpers.
//!
//! Internal storage always uses SI base units (metres, square metres, cubic
//! metres, hours, kilograms, count) so that arithmetic is unambiguous. Named
//! construction units — linear feet (LF), square feet (SF), square yards (SY),
//! cubic yards (CY), each, hour/day/week, kilogram, ton, and their metric
//! counterparts — are converted at the boundary. Also provides waste-factor
//! application, bank/loose/compacted soil-volume conversion, rebar weight, and
//! paint coverage helpers.

use serde::{Deserialize, Serialize};

const M_PER_FT: f64 = 0.3048;
const M2_PER_SF: f64 = 0.09290304;
const M2_PER_SY: f64 = 0.83612736;
const M3_PER_CY: f64 = 0.764554857_984;
const M_PER_YD: f64 = 0.9144;
const FT3_PER_CY: f64 = 27.0;
const KG_PER_SHORT_TON: f64 = 907.18474;

/// A linear measure, stored in metres.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Length(pub f64);

impl Length {
    /// Length in metres.
    pub fn meters(self) -> f64 {
        self.0
    }
    /// Length in feet (LF).
    pub fn feet(self) -> f64 {
        self.0 / M_PER_FT
    }
    /// Length in inches.
    pub fn inches(self) -> f64 {
        self.0 / M_PER_FT * 12.0
    }
    /// Length in yards.
    pub fn yards(self) -> f64 {
        self.0 / M_PER_YD
    }
    /// Build from metres.
    pub fn from_meters(v: f64) -> Self {
        Self(v)
    }
    /// Build from feet (LF).
    pub fn from_feet(v: f64) -> Self {
        Self(v * M_PER_FT)
    }
    /// Build from inches.
    pub fn from_inches(v: f64) -> Self {
        Self(v * M_PER_FT / 12.0)
    }
    /// Build from yards.
    pub fn from_yards(v: f64) -> Self {
        Self(v * M_PER_YD)
    }
}

/// An area measure, stored in square metres.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Area(pub f64);

impl Area {
    /// Area in square metres (m²).
    pub fn square_meters(self) -> f64 {
        self.0
    }
    /// Area in square feet (SF).
    pub fn square_feet(self) -> f64 {
        self.0 / M2_PER_SF
    }
    /// Area in square yards (SY).
    pub fn square_yards(self) -> f64 {
        self.0 / M2_PER_SY
    }
    /// Build from square metres.
    pub fn from_square_meters(v: f64) -> Self {
        Self(v)
    }
    /// Build from square feet (SF).
    pub fn from_square_feet(v: f64) -> Self {
        Self(v * M2_PER_SF)
    }
    /// Build from square yards (SY).
    pub fn from_square_yards(v: f64) -> Self {
        Self(v * M2_PER_SY)
    }
}

/// A volume measure, stored in cubic metres.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Volume(pub f64);

impl Volume {
    /// Volume in cubic metres (m³).
    pub fn cubic_meters(self) -> f64 {
        self.0
    }
    /// Volume in cubic yards (CY).
    pub fn cubic_yards(self) -> f64 {
        self.0 / M3_PER_CY
    }
    /// Volume in cubic feet.
    pub fn cubic_feet(self) -> f64 {
        self.0 / (M3_PER_CY / FT3_PER_CY)
    }
    /// Build from cubic metres.
    pub fn from_cubic_meters(v: f64) -> Self {
        Self(v)
    }
    /// Build from cubic yards (CY).
    pub fn from_cubic_yards(v: f64) -> Self {
        Self(v * M3_PER_CY)
    }
    /// Build from cubic feet.
    pub fn from_cubic_feet(v: f64) -> Self {
        Self(v * (M3_PER_CY / FT3_PER_CY))
    }
}

/// A count measure, stored as a dimensionless quantity ("each").
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Count(pub f64);

impl Count {
    /// The raw count ("each").
    pub fn each(self) -> f64 {
        self.0
    }
    /// Build from a count.
    pub fn from_each(v: f64) -> Self {
        Self(v)
    }
}

/// A time measure, stored in hours.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration(pub f64);

impl Duration {
    /// Duration in hours.
    pub fn hours(self) -> f64 {
        self.0
    }
    /// Duration in days (24 h).
    pub fn days(self) -> f64 {
        self.0 / 24.0
    }
    /// Duration in weeks (168 h).
    pub fn weeks(self) -> f64 {
        self.0 / (24.0 * 7.0)
    }
    /// Build from hours.
    pub fn from_hours(v: f64) -> Self {
        Self(v)
    }
    /// Build from days.
    pub fn from_days(v: f64) -> Self {
        Self(v * 24.0)
    }
    /// Build from weeks.
    pub fn from_weeks(v: f64) -> Self {
        Self(v * 24.0 * 7.0)
    }
}

/// A mass measure, stored in kilograms.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Mass(pub f64);

impl Mass {
    /// Mass in kilograms.
    pub fn kilograms(self) -> f64 {
        self.0
    }
    /// Mass in US short tons (2,000 lb).
    pub fn tons(self) -> f64 {
        self.0 / KG_PER_SHORT_TON
    }
    /// Mass in metric tonnes (1,000 kg).
    pub fn metric_tonnes(self) -> f64 {
        self.0 / 1_000.0
    }
    /// Build from kilograms.
    pub fn from_kilograms(v: f64) -> Self {
        Self(v)
    }
    /// Build from US short tons.
    pub fn from_tons(v: f64) -> Self {
        Self(v * KG_PER_SHORT_TON)
    }
    /// Build from metric tonnes.
    pub fn from_metric_tonnes(v: f64) -> Self {
        Self(v * 1_000.0)
    }
}

macro_rules! measure_binop {
    ($t:ident, $op:ident, $fn:ident) => {
        impl std::ops::$op for $t {
            type Output = $t;
            fn $fn(self, rhs: $t) -> $t {
                $t(self.0.$fn(rhs.0))
            }
        }
    };
}

measure_binop!(Length, Add, add);
measure_binop!(Length, Sub, sub);
measure_binop!(Area, Add, add);
measure_binop!(Area, Sub, sub);
measure_binop!(Volume, Add, add);
measure_binop!(Volume, Sub, sub);
measure_binop!(Count, Add, add);
measure_binop!(Count, Sub, sub);
measure_binop!(Duration, Add, add);
measure_binop!(Duration, Sub, sub);
measure_binop!(Mass, Add, add);
measure_binop!(Mass, Sub, sub);

/// A multiplicative waste factor expressed as a ratio (0.10 = 10% added).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WasteFactor(pub f64);

impl WasteFactor {
    /// A factor of zero (no waste).
    pub fn none() -> Self {
        Self(0.0)
    }
    /// Build from a percentage (10.0 → 0.10).
    pub fn from_percent(pct: f64) -> Self {
        Self(pct / 100.0)
    }
    /// The underlying ratio.
    pub fn ratio(self) -> f64 {
        self.0
    }
    /// Apply the waste factor to a quantity, returning the gross amount.
    pub fn apply<T: Wasteable>(self, q: T) -> T {
        q.apply_waste(self)
    }
}

/// Quantities that understand waste-factor expansion.
pub trait Wasteable {
    /// Return `self` scaled by `(1 + waste.ratio())`.
    fn apply_waste(self, waste: WasteFactor) -> Self;
}

macro_rules! impl_wasteable {
    ($t:ident) => {
        impl Wasteable for $t {
            fn apply_waste(self, waste: WasteFactor) -> Self {
                $t(self.0 * (1.0 + waste.ratio()))
            }
        }
    };
}

impl_wasteable!(Length);
impl_wasteable!(Area);
impl_wasteable!(Volume);
impl_wasteable!(Count);
impl_wasteable!(Mass);

/// Soil volume state. Bank = in-situ, Loose = excavated/swelled,
/// Compacted = re-compacted/shrunk.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoilState {
    /// In-situ, undisturbed volume.
    Bank,
    /// Excavated volume after swell.
    Loose,
    /// Re-compacted volume after shrinkage.
    Compacted,
}

/// A soil volume annotated with its state, supporting swell/shrink conversion.
///
/// Conversions follow: `loose = bank * (1 + swell)` and
/// `compacted = bank * (1 - shrink)`, where swell/shrink are ratios.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoilVolume {
    /// Volume magnitude in cubic metres.
    pub volume: Volume,
    /// State of the stored volume.
    pub state: SoilState,
}

impl SoilVolume {
    /// Create a soil volume in the given state.
    pub fn new(volume: Volume, state: SoilState) -> Self {
        Self { volume, state }
    }

    /// Volume in cubic yards, in the current state.
    pub fn cubic_yards(&self) -> f64 {
        self.volume.cubic_yards()
    }

    /// Convert to another state given swell and shrink ratios.
    pub fn convert(self, target: SoilState, swell: f64, shrink: f64) -> Self {
        if self.state == target {
            return self;
        }
        let bank = match self.state {
            SoilState::Bank => self.volume.0,
            SoilState::Loose => self.volume.0 / (1.0 + swell),
            SoilState::Compacted => self.volume.0 / (1.0 - shrink),
        };
        let out = match target {
            SoilState::Bank => bank,
            SoilState::Loose => bank * (1.0 + swell),
            SoilState::Compacted => bank * (1.0 - shrink),
        };
        Self::new(Volume::from_cubic_meters(out), target)
    }
}

/// UNIT mass of reinforcing steel per metre for common US bar sizes (#3–#11).
pub fn rebar_unit_weight_kg_per_m(size: u8) -> Option<f64> {
    let kg_per_m = match size {
        3 => 0.561,
        4 => 0.996,
        5 => 1.556,
        6 => 2.240,
        7 => 3.049,
        8 => 3.982,
        9 => 5.060,
        10 => 6.404,
        11 => 7.907,
        _ => return None,
    };
    Some(kg_per_m)
}

/// Total rebar weight for `length` of bar `size`.
pub fn rebar_weight(length: Length, size: u8) -> Option<Mass> {
    let per_m = rebar_unit_weight_kg_per_m(size)?;
    Some(Mass::from_kilograms(length.meters() * per_m))
}

/// Paint coverage: area achievable per unit volume of coating.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaintCoverage {
    /// Square metres covered per litre of paint.
    pub sqm_per_litre: f64,
}

impl PaintCoverage {
    /// Build a coverage rate.
    pub fn new(sqm_per_litre: f64) -> Self {
        Self { sqm_per_litre }
    }
    /// Area covered by `litres` of paint at this coverage rate.
    pub fn covered_area(self, litres: f64) -> Area {
        Area::from_square_meters(litres * self.sqm_per_litre)
    }
    /// Litres required to cover `area`.
    pub fn litres_for(self, area: Area) -> f64 {
        area.square_meters() / self.sqm_per_litre
    }
}

/// Round `value` to `decimals` decimal places (half away from zero).
pub fn round_to_decimals(value: f64, decimals: u32) -> f64 {
    let factor = 10f64.powi(decimals as i32);
    let signed = value.signum();
    let abs = value.abs() * factor;
    let rounded = (abs + 0.5).floor();
    signed * rounded / factor
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_roundtrip() {
        let l = Length::from_feet(10.0);
        assert!((l.meters() - 3.048).abs() < 1e-9);
        assert!((l.feet() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn area_conversions() {
        let a = Area::from_square_yards(1.0);
        assert!((a.square_feet() - 9.0).abs() < 1e-6);
    }

    #[test]
    fn volume_cy_to_m3() {
        let v = Volume::from_cubic_yards(1.0);
        assert!((v.cubic_meters() - M3_PER_CY).abs() < 1e-9);
    }

    #[test]
    fn waste_factor() {
        let w = WasteFactor::from_percent(10.0);
        let q = w.apply(Area::from_square_feet(100.0));
        assert!((q.square_feet() - 110.0).abs() < 1e-9);
    }

    #[test]
    fn soil_swell_shrink() {
        let bank = SoilVolume::new(Volume::from_cubic_yards(100.0), SoilState::Bank);
        let loose = bank.convert(SoilState::Loose, 0.25, 0.12);
        assert!((loose.cubic_yards() - 125.0).abs() < 1e-6);
        let compacted = bank.convert(SoilState::Compacted, 0.25, 0.12);
        assert!((compacted.cubic_yards() - 88.0).abs() < 1e-6);
    }

    #[test]
    fn rebar_and_paint() {
        let w = rebar_weight(Length::from_meters(12.0), 5).unwrap();
        assert!((w.kilograms() - 12.0 * 1.556).abs() < 1e-6);
        let cov = PaintCoverage::new(10.0);
        assert!((cov.covered_area(2.0).square_meters() - 20.0).abs() < 1e-9);
        assert!((cov.litres_for(Area::from_square_meters(50.0)) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn rounding() {
        assert_eq!(round_to_decimals(2.345, 2), 2.35);
        assert_eq!(round_to_decimals(-2.345, 2), -2.35);
    }
}
