// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The takeoff engine: converts a [`Project`] model into per-element quantities.

use tpt_c_classification::Classification;
use tpt_c_core::ElementId;
use tpt_c_model::{Element, Project};
use tpt_c_units::WasteFactor;

use crate::rules::{concrete_volume, formwork_area, paint_area, rebar_weight_for, QuantityRules};
use crate::{MeasuredQuantity, QuantityKind, TakeoffQuantity};

/// Drives quantity extraction over a [`Project`] using a [`QuantityRules`] set.
#[derive(Clone, Debug)]
pub struct TakeoffEngine {
    rules: QuantityRules,
}

impl Default for TakeoffEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TakeoffEngine {
    /// An engine using the default waste rules.
    pub fn new() -> Self {
        Self {
            rules: QuantityRules::new(),
        }
    }

    /// An engine with an explicit rule set (e.g. project-specific waste rates).
    pub fn with_rules(rules: QuantityRules) -> Self {
        Self { rules }
    }

    /// Run the takeoff over every element in `project`.
    pub fn run(&self, project: &Project) -> TakeoffResult {
        let mut items = Vec::new();
        for element in &project.elements {
            items.push(self.takeoff_element(element));
        }
        TakeoffResult {
            items,
            rules: self.rules.clone(),
        }
    }

    fn takeoff_element(&self, element: &Element) -> TakeoffItem {
        let mut quantities: Vec<TakeoffQuantity> = Vec::new();

        for qs in &element.quantity_sets {
            for nq in &qs.quantities {
                if let Some(mq) = MeasuredQuantity::from_model(&nq.quantity) {
                    quantities.push(TakeoffQuantity {
                        name: box_name(&nq.name),
                        net: mq,
                        waste: self.rules.waste_for_kind(&element.category, mq.kind()),
                        manual: false,
                    });
                }
            }
        }

        let has = |qs: &[TakeoffQuantity], n: &str| {
            qs.iter().any(|q| q.name.eq_ignore_ascii_case(n))
        };

        if let Some(v) = concrete_volume(element) {
            if !has(&quantities, "ConcreteVolume") {
                quantities.push(TakeoffQuantity {
                    name: "ConcreteVolume",
                    net: MeasuredQuantity::Volume(v),
                    waste: self.rules.concrete_waste(&element.category),
                    manual: false,
                });
            }
        }
        if let Some(a) = formwork_area(element) {
            if !has(&quantities, "FormworkArea") {
                quantities.push(TakeoffQuantity {
                    name: "FormworkArea",
                    net: MeasuredQuantity::Area(a),
                    waste: self.rules.formwork_waste(&element.category),
                    manual: false,
                });
            }
        }
        if let Some(m) = rebar_weight_for(element) {
            if !has(&quantities, "RebarWeight") {
                quantities.push(TakeoffQuantity {
                    name: "RebarWeight",
                    net: MeasuredQuantity::Mass(m),
                    waste: self.rules.rebar_waste(&element.category),
                    manual: false,
                });
            }
        }
        if let Some(a) = paint_area(element) {
            if !has(&quantities, "PaintArea") {
                quantities.push(TakeoffQuantity {
                    name: "PaintArea",
                    net: MeasuredQuantity::Area(a),
                    waste: self.rules.paint_waste(&element.category),
                    manual: false,
                });
            }
        }

        TakeoffItem {
            element_id: element.id,
            name: element.name.clone(),
            category: element.category.clone(),
            classification: element.classification.clone(),
            quantities,
        }
    }
}

/// One element's extracted quantities.
#[derive(Clone, Debug)]
pub struct TakeoffItem {
    /// The source element id.
    pub element_id: ElementId,
    /// Element name.
    pub name: String,
    /// Element category.
    pub category: String,
    /// Classification attached to the element (if any).
    pub classification: Option<Classification>,
    /// Extracted net/gross quantities.
    pub quantities: Vec<TakeoffQuantity>,
}

/// The full result of a takeoff pass over a project.
#[derive(Clone, Debug)]
pub struct TakeoffResult {
    /// Per-element quantities.
    pub items: Vec<TakeoffItem>,
    /// The rules used to produce this result.
    pub rules: QuantityRules,
}

impl TakeoffResult {
    /// Sum of net quantities of a given kind (in base units).
    pub fn total_net(&self, kind: QuantityKind) -> f64 {
        self.items
            .iter()
            .flat_map(|i| &i.quantities)
            .filter(|q| q.net.kind() == kind)
            .map(|q| q.net.base_value())
            .sum()
    }

    /// Sum of gross quantities of a given kind (net expanded by waste).
    pub fn total_gross(&self, kind: QuantityKind) -> f64 {
        self.items
            .iter()
            .flat_map(|i| &i.quantities)
            .filter(|q| q.net.kind() == kind)
            .map(|q| q.gross().base_value())
            .sum()
    }

    /// Override a single named quantity on an element with an estimator value.
    ///
    /// The overridden quantity is flagged `manual`, which downstream pricing
    /// should prefer over any model-derived value.
    pub fn override_quantity(
        &mut self,
        element_id: ElementId,
        name: &str,
        value: MeasuredQuantity,
    ) -> bool {
        let item = match self.items.iter_mut().find(|i| i.element_id == element_id) {
            Some(i) => i,
            None => return false,
        };
        let entry = item
            .quantities
            .iter_mut()
            .find(|q| q.name.eq_ignore_ascii_case(name));
        match entry {
            Some(q) => {
                q.net = value;
                q.manual = true;
                true
            }
            None => {
                item.quantities.push(TakeoffQuantity {
                    name: box_name(name),
                    net: value,
                    waste: item
                        .classification
                        .as_ref()
                        .map(|_| WasteFactor::none())
                        .unwrap_or(WasteFactor::none()),
                    manual: true,
                });
                true
            }
        }
    }
}

/// Leak a `&str` name into a `&'static str` for `TakeoffQuantity`.
///
/// Takeoff item names are a small fixed vocabulary; we intern them via `Box` and
/// leak intentionally. Quantities are short-lived within a single takeoff pass
/// and the count is bounded by the number of distinct names in a model.
fn box_name(name: &str) -> &'static str {
    Box::leak(name.to_string().into_boxed_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_classification::{Classification, ClassificationSystem};
    use tpt_c_core::ProjectId;
    use tpt_c_ids::IdFactory;
    use tpt_c_model::{PropertySet, PropertyValue, Quantity, QuantitySet};
    use tpt_c_units::{Area, Length, Volume, WasteFactor};

    #[test]
    fn engine_extracts_and_derives() {
        let mut p = Project::new(ProjectId::nil(), "Demo");
        let mut e = Element::new(IdFactory::element(), "W1", "Wall")
            .classified(Classification::new(ClassificationSystem::MasterFormat, "03 30 00"))
            .with_quantity_set(
                QuantitySet::new("BaseQuantities")
                    .with("Length", Quantity::Length(Length::from_feet(40.0)))
                    .with("GrossVolume", Quantity::Volume(Volume::from_cubic_yards(5.0))),
            );
        e.property_sets.push(PropertySet::new("R").with("RebarSize", PropertyValue::Number(5.0)));
        e.quantity_sets.push(
            QuantitySet::new("Q").with("RebarLength", Quantity::Length(Length::from_meters(10.0))),
        );
        p.add_element(e);

        let result = TakeoffEngine::new().run(&p);
        let item = &result.items[0];
        let names: Vec<&str> = item.quantities.iter().map(|q| q.name).collect();
        assert!(names.contains(&"ConcreteVolume"));
        assert!(names.contains(&"RebarWeight"));
        assert!(names.contains(&"FormworkArea"));
        assert!(result.total_gross(QuantityKind::Volume) > result.total_net(QuantityKind::Volume));
    }

    #[test]
    fn manual_override_replaces_net() {
        let mut p = Project::new(ProjectId::nil(), "Demo");
        let id = IdFactory::element();
        p.add_element(
            Element::new(id, "S1", "Slab").with_quantity_set(
                QuantitySet::new("Q").with("GrossVolume", Quantity::Volume(Volume::from_cubic_yards(2.0))),
            ),
        );
        let mut result = TakeoffEngine::new().run(&p);
        assert!(result.override_quantity(id, "GrossVolume", MeasuredQuantity::Volume(Volume::from_cubic_yards(9.0))));
        let item = result.items.iter().find(|i| i.element_id == id).unwrap();
        let q = item.quantities.iter().find(|q| q.name == "ConcreteVolume").unwrap();
        assert!(q.manual);
        assert!((result.total_net(QuantityKind::Volume) - Volume::from_cubic_yards(9.0).cubic_meters()).abs() < 1e-9);
    }

    #[test]
    fn waste_for_kind_mapping() {
        let r = QuantityRules::new();
        assert_eq!(r.waste_for_kind("Wall", QuantityKind::Volume), r.concrete_waste("Wall"));
        assert_eq!(r.waste_for_kind("Wall", QuantityKind::Area), r.paint_waste("Wall"));
        assert_eq!(r.waste_for_kind("Wall", QuantityKind::Mass), r.rebar_waste("Wall"));
        let _ = WasteFactor::none();
        let _ = Area::from_square_feet(0.0);
    }
}
