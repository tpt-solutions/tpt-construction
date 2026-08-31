// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Derived quantity rules and default waste factors.
//!
//! Parsers (IFC, glTF, ...) populate an element's [`QuantitySet`]s, but some
//! estimating quantities are derived rather than stored: concrete volume from a
//! cross-section, formwork area from a perimeter, rebar weight from bar length
//! and size, paint area from finish area, and flooring area net of deductions.

use tpt_c_model::{Element, PropertyValue, Quantity};
use tpt_c_units::{Area, Length, Mass, Volume, WasteFactor};

/// Default waste factor (ratio) for cast-in-place concrete volume.
pub const DEFAULT_WASTE_CONCRETE: f64 = 0.07;
/// Default waste factor (ratio) for formwork area.
pub const DEFAULT_WASTE_FORMWORK: f64 = 0.10;
/// Default waste factor (ratio) for rebar weight.
pub const DEFAULT_WASTE_REBAR: f64 = 0.03;
/// Default waste factor (ratio) for paint / finish area.
pub const DEFAULT_WASTE_PAINT: f64 = 0.10;
/// Default waste factor (ratio) for flooring area.
pub const DEFAULT_WASTE_FLOORING: f64 = 0.05;

/// A configureable set of takeoff rules: per-category waste overrides plus the
/// shared defaults.
#[derive(Clone, Debug)]
pub struct QuantityRules {
    /// Overrides keyed by element category substring (matched case-insensitively).
    concrete_overrides: Vec<(String, f64)>,
    formwork_overrides: Vec<(String, f64)>,
    rebar_overrides: Vec<(String, f64)>,
    paint_overrides: Vec<(String, f64)>,
    flooring_overrides: Vec<(String, f64)>,
}

impl Default for QuantityRules {
    fn default() -> Self {
        Self {
            concrete_overrides: Vec::new(),
            formwork_overrides: Vec::new(),
            rebar_overrides: Vec::new(),
            paint_overrides: Vec::new(),
            flooring_overrides: Vec::new(),
        }
    }
}

impl QuantityRules {
    /// A rule set using only the global defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the concrete waste factor for elements whose category contains
    /// `category` (case-insensitive).
    pub fn with_concrete_waste(mut self, category: impl Into<String>, ratio: f64) -> Self {
        self.concrete_overrides.push((category.into().to_lowercase(), ratio));
        self
    }

    /// Override the formwork waste factor for a category.
    pub fn with_formwork_waste(mut self, category: impl Into<String>, ratio: f64) -> Self {
        self.formwork_overrides.push((category.into().to_lowercase(), ratio));
        self
    }

    /// Override the rebar waste factor for a category.
    pub fn with_rebar_waste(mut self, category: impl Into<String>, ratio: f64) -> Self {
        self.rebar_overrides.push((category.into().to_lowercase(), ratio));
        self
    }

    /// Override the paint waste factor for a category.
    pub fn with_paint_waste(mut self, category: impl Into<String>, ratio: f64) -> Self {
        self.paint_overrides.push((category.into().to_lowercase(), ratio));
        self
    }

    /// Override the flooring waste factor for a category.
    pub fn with_flooring_waste(mut self, category: impl Into<String>, ratio: f64) -> Self {
        self.flooring_overrides.push((category.into().to_lowercase(), ratio));
        self
    }

    fn pick(overrides: &[(String, f64)], category: &str, default: f64) -> f64 {
        let cat = category.to_lowercase();
        overrides
            .iter()
            .find(|(needle, _)| cat.contains(needle.as_str()))
            .map(|(_, v)| *v)
            .unwrap_or(default)
    }

    /// Waste factor for concrete volume on `category`.
    pub fn concrete_waste(&self, category: &str) -> WasteFactor {
        WasteFactor(Self::pick(&self.concrete_overrides, category, DEFAULT_WASTE_CONCRETE))
    }

    /// Waste factor for formwork area on `category`.
    pub fn formwork_waste(&self, category: &str) -> WasteFactor {
        WasteFactor(Self::pick(&self.formwork_overrides, category, DEFAULT_WASTE_FORMWORK))
    }

    /// Waste factor for rebar weight on `category`.
    pub fn rebar_waste(&self, category: &str) -> WasteFactor {
        WasteFactor(Self::pick(&self.rebar_overrides, category, DEFAULT_WASTE_REBAR))
    }

    /// Waste factor for paint area on `category`.
    pub fn paint_waste(&self, category: &str) -> WasteFactor {
        WasteFactor(Self::pick(&self.paint_overrides, category, DEFAULT_WASTE_PAINT))
    }

    /// Waste factor for flooring area on `category`.
    pub fn flooring_waste(&self, category: &str) -> WasteFactor {
        WasteFactor(Self::pick(&self.flooring_overrides, category, DEFAULT_WASTE_FLOORING))
    }

    /// Default waste factor for a quantity kind on a category, used when a stored
    /// model quantity has no more specific semantic.
    pub fn waste_for_kind(&self, category: &str, kind: crate::QuantityKind) -> WasteFactor {
        match kind {
            crate::QuantityKind::Count => self.flooring_waste(category),
            crate::QuantityKind::Length => self.formwork_waste(category),
            crate::QuantityKind::Area => self.paint_waste(category),
            crate::QuantityKind::Volume => self.concrete_waste(category),
            crate::QuantityKind::Mass => self.rebar_waste(category),
        }
    }
}

fn find_quantity(element: &Element, names: &[&str]) -> Option<Quantity> {
    for qs in &element.quantity_sets {
        for nq in &qs.quantities {
            if names.iter().any(|n| nq.name.eq_ignore_ascii_case(n)) {
                return Some(nq.quantity);
            }
        }
    }
    None
}

fn find_property_number(element: &Element, names: &[&str]) -> Option<f64> {
    for ps in &element.property_sets {
        for p in &ps.properties {
            if names.iter().any(|n| p.name.eq_ignore_ascii_case(n)) {
                if let PropertyValue::Number(v) = p.value {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// Concrete volume for an element.
///
/// Prefers an explicit stored volume quantity (`GrossVolume`, `NetVolume`,
/// `Volume`, `ConcreteVolume`); otherwise estimates it from a length quantity
/// times a `CrossSectionArea` property (both in base units).
pub fn concrete_volume(element: &Element) -> Option<Volume> {
    if let Some(Quantity::Volume(v)) = find_quantity(element, &["GrossVolume", "NetVolume", "Volume", "ConcreteVolume"]) {
        return Some(v);
    }
    let len = match find_quantity(element, &["Length"]) {
        Some(Quantity::Length(l)) => l.meters(),
        _ => return None,
    };
    let csa = find_property_number(element, &["CrossSectionArea"])?;
    Some(Volume::from_cubic_meters(len * csa))
}

/// Formwork (contact) area for an element.
///
/// Prefers a stored area quantity (`FormworkArea`, `SurfaceArea`, `Area`);
/// otherwise estimates it from a length quantity times a `Perimeter` property.
pub fn formwork_area(element: &Element) -> Option<Area> {
    if let Some(Quantity::Area(a)) = find_quantity(element, &["FormworkArea", "SurfaceArea", "Area"]) {
        return Some(a);
    }
    let len = match find_quantity(element, &["Length"]) {
        Some(Quantity::Length(l)) => l.meters(),
        _ => return None,
    };
    let perim = find_property_number(element, &["Perimeter"])?;
    Some(Area::from_square_meters(len * perim))
}

/// Rebar weight for an element, derived from a `RebarLength` quantity (or
/// property) and a `RebarSize` property (US bar number, #3–#11).
pub fn rebar_weight_for(element: &Element) -> Option<Mass> {
    let size = find_property_number(element, &["RebarSize"])? as u8;
    let len = match find_quantity(element, &["RebarLength"]) {
        Some(Quantity::Length(l)) => l,
        _ => {
            let m = find_property_number(element, &["RebarLength"])?;
            Length::from_meters(m)
        }
    };
    tpt_c_units::rebar_weight(len, size)
}

/// Paint / finish area for an element.
///
/// Prefers a stored area quantity (`PaintArea`, `FinishArea`, `Area`).
pub fn paint_area(element: &Element) -> Option<Area> {
    if let Some(Quantity::Area(a)) = find_quantity(element, &["PaintArea", "FinishArea", "Area"]) {
        return Some(a);
    }
    None
}

/// Net flooring area after subtracting a set of deductions (openings, fixtures).
pub fn flooring_area_with_deductions(floor: Area, deductions: &[Area]) -> Area {
    let total: f64 = deductions.iter().map(|d| d.square_meters()).sum();
    Area::from_square_meters((floor.square_meters() - total).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::ElementId;
    use tpt_c_model::{PropertySet, Quantity, QuantitySet};
    use tpt_c_units::{Area, Length, Volume};

    fn wall() -> Element {
        Element::new(ElementId::nil(), "W1", "Wall").with_quantity_set(
            QuantitySet::new("BaseQuantities")
                .with("Length", Quantity::Length(Length::from_feet(40.0)))
                .with("GrossVolume", Quantity::Volume(Volume::from_cubic_yards(5.0))),
        )
    }

    #[test]
    fn concrete_volume_prefers_stored() {
        assert!((concrete_volume(&wall()).unwrap().cubic_yards() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn concrete_volume_derived_from_csa() {
        let mut e = Element::new(ElementId::nil(), "C1", "Column")
            .with_quantity_set(QuantitySet::new("Q").with("Length", Quantity::Length(Length::from_meters(3.0))));
        e.property_sets.push(
            PropertySet::new("Rebar")
                .with("CrossSectionArea", PropertyValue::Number(0.2)),
        );
        let v = concrete_volume(&e).unwrap();
        assert!((v.cubic_meters() - 0.6).abs() < 1e-9);
    }

    #[test]
    fn rebar_weight_rule() {
        let mut e = Element::new(ElementId::nil(), "B1", "Beam");
        e.property_sets.push(PropertySet::new("R").with("RebarSize", PropertyValue::Number(5.0)));
        e.quantity_sets.push(
            QuantitySet::new("Q").with("RebarLength", Quantity::Length(Length::from_meters(10.0))),
        );
        let m = rebar_weight_for(&e).unwrap();
        assert!((m.kilograms() - 10.0 * 1.556).abs() < 1e-6);
    }

    #[test]
    fn flooring_deductions() {
        let net = flooring_area_with_deductions(
            Area::from_square_feet(1000.0),
            &[Area::from_square_feet(100.0), Area::from_square_feet(50.0)],
        );
        assert!((net.square_feet() - 850.0).abs() < 1e-6);
    }

    #[test]
    fn rule_overrides() {
        let r = QuantityRules::new().with_concrete_waste("slab", 0.12);
        assert!((r.concrete_waste("Slab-On-Grade").ratio() - 0.12).abs() < 1e-9);
        assert!((r.concrete_waste("Wall").ratio() - DEFAULT_WASTE_CONCRETE).abs() < 1e-9);
    }
}
