// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Horizontal and vertical road/rail alignments, curves, superelevation,
//! stationing, and corridor modeling.
//!
//! Provides types for horizontal alignments (tangents, circular curves,
//! spirals), vertical alignments (grades, parabolic vertical curves),
//! stationing along a baseline, and superelevation runoff.

use serde::{Deserialize, Serialize};
use tpt_c_units::Length;

/// A point in 2D horizontal space.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    /// Easting / x coordinate (metres).
    pub x: f64,
    /// Northing / y coordinate (metres).
    pub y: f64,
}

impl Point2D {
    /// Build a 2D point.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Distance from this point to another.
    pub fn distance_to(&self, other: &Point2D) -> Length {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        Length::from_meters((dx * dx + dy * dy).sqrt())
    }
}

/// A horizontal alignment element.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HorizontalElement {
    /// Straight tangent between two points.
    Tangent {
        /// Start point.
        start: Point2D,
        /// End point.
        end: Point2D,
    },
    /// Circular curve with constant radius.
    Circular {
        /// Curve start point.
        start: Point2D,
        /// Center point.
        center: Point2D,
        /// Radius in metres (positive = left turn).
        radius: f64,
        /// Sweep angle in radians (positive = left turn).
        sweep: f64,
    },
    /// Transition spiral (clothoid approximation).
    Spiral {
        /// Start point.
        start: Point2D,
        /// Start tangent direction (radians, clockwise from north/east convention as needed).
        start_angle: f64,
        /// End tangent direction.
        end_angle: f64,
        /// Length of spiral in metres.
        length: f64,
    },
}

impl HorizontalElement {
    /// Horizontal length of this element in metres.
    pub fn length(&self) -> Length {
        match self {
            HorizontalElement::Tangent { start, end } => start.distance_to(end),
            HorizontalElement::Circular { radius, sweep, .. } => {
                Length::from_meters(radius * sweep.abs())
            }
            HorizontalElement::Spiral { length, .. } => Length::from_meters(*length),
        }
    }
}

/// A vertical alignment element.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VerticalElement {
    /// Constant grade between two elevations.
    Grade {
        /// Start elevation (metres).
        start_elevation: f64,
        /// Grade as a ratio (0.03 = +3%).
        grade: f64,
        /// Length in metres.
        length: f64,
    },
    /// Parabolic vertical curve.
    VerticalCurve {
        /// Start elevation.
        start_elevation: f64,
        /// Incoming grade.
        grade_in: f64,
        /// Outgoing grade.
        grade_out: f64,
        /// Length in metres.
        length: f64,
    },
}

impl VerticalElement {
    /// Elevation at distance `d` along the element (0 <= d <= length).
    pub fn elevation_at(&self, d: f64) -> f64 {
        match self {
            VerticalElement::Grade {
                start_elevation,
                grade,
                length,
            } => {
                let _t = (d / length).clamp(0.0, 1.0);
                start_elevation + grade * d
            }
            VerticalElement::VerticalCurve {
                start_elevation,
                grade_in,
                grade_out,
                length,
            } => {
                let _t = (d / length).clamp(0.0, 1.0);
                let a = grade_in;
                let b = (grade_out - grade_in) / length;
                start_elevation + a * d + b * d * d
            }
        }
    }

    /// Length of the element in metres.
    pub fn length(&self) -> Length {
        match self {
            VerticalElement::Grade { length, .. } => Length::from_meters(*length),
            VerticalElement::VerticalCurve { length, .. } => Length::from_meters(*length),
        }
    }
}

/// A station along an alignment ( metres from a datum, typically 1000 = 10+00).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Station(pub i64);

impl Station {
    /// Build a station from raw metres.
    pub fn from_metres(metres: i64) -> Self {
        Self(metres)
    }

    /// Raw metres.
    pub fn metres(&self) -> i64 {
        self.0
    }

    /// Station formatted as `km+00.0`.
    pub fn format(&self) -> String {
        let k = self.0 / 1000;
        let r = self.0 % 1000;
        format!("{k}+{r:.1}")
    }
}

/// Superelevation (cross-slope) configuration.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Superelevation {
    /// Maximum superelevation rate (e.g. 0.06 = 6%).
    pub max_rate: f64,
    /// Runoff length in metres.
    pub runoff_length: f64,
    /// Normal crown slope (e.g. -0.02).
    pub crown_slope: f64,
}

impl Superelevation {
    /// Build a superelevation definition.
    pub fn new(max_rate: f64, runoff_length: f64, crown_slope: f64) -> Self {
        Self {
            max_rate,
            runoff_length,
            crown_slope,
        }
    }

    /// Superelevation rate at distance `d` into the runoff (linear transition).
    pub fn rate_at(&self, d: f64) -> f64 {
        let t = (d / self.runoff_length).clamp(0.0, 1.0);
        self.crown_slope + (self.max_rate - self.crown_slope) * t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_distance() {
        let a = Point2D::new(0.0, 0.0);
        let b = Point2D::new(3.0, 4.0);
        assert_eq!(a.distance_to(&b).meters(), 5.0);
    }

    #[test]
    fn tangent_length() {
        let el = HorizontalElement::Tangent {
            start: Point2D::new(0.0, 0.0),
            end: Point2D::new(30.0, 40.0),
        };
        assert_eq!(el.length().meters(), 50.0);
    }

    #[test]
    fn circular_length() {
        let el = HorizontalElement::Circular {
            start: Point2D::new(0.0, 0.0),
            center: Point2D::new(0.0, 0.0),
            radius: 100.0,
            sweep: std::f64::consts::FRAC_PI_2,
        };
        assert_eq!(el.length().meters(), 100.0 * std::f64::consts::FRAC_PI_2);
    }

    #[test]
    fn vertical_grade_elevation() {
        let el = VerticalElement::Grade {
            start_elevation: 100.0,
            grade: 0.03,
            length: 100.0,
        };
        assert_eq!(el.elevation_at(0.0), 100.0);
        assert_eq!(el.elevation_at(100.0), 103.0);
    }

    #[test]
    fn vertical_curve_elevation() {
        let el = VerticalElement::VerticalCurve {
            start_elevation: 100.0,
            grade_in: 0.03,
            grade_out: -0.02,
            length: 100.0,
        };
        assert_eq!(el.elevation_at(0.0), 100.0);
        assert_eq!(el.elevation_at(50.0), 100.0 + 0.03 * 50.0 + (-0.05 / 100.0) * 2500.0);
    }

    #[test]
    fn station_format() {
        assert_eq!(Station::from_metres(10500).format(), "10+500.0");
        assert_eq!(Station::from_metres(12340).format(), "12+340.0");
    }

    #[test]
    fn superelevation_transition() {
        let se = Superelevation::new(0.06, 100.0, -0.02);
        assert!((se.rate_at(0.0) - (-0.02)).abs() < 1e-9);
        assert!((se.rate_at(100.0) - 0.06).abs() < 1e-9);
        assert!((se.rate_at(50.0) - 0.02).abs() < 1e-9);
    }
}
