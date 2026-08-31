// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The neutral construction domain model.
//!
//! File-format parsers (IFC, glTF, ...) and domain engines (takeoff, cost,
//! scheduling) all speak this model. It is serializable, deterministic, and
//! free of any IO. Geometry is referenced opaquely; rich geometric processing
//! lives in `tpt-c-geometry`.

use serde::{Deserialize, Serialize};
use tpt_c_classification::{Classification, ClassificationSystem};
use tpt_c_core::{ElementId, Identified, ProjectId};
use tpt_c_ids::ExternalId;
use tpt_c_units::{Area, Count, Duration, Length, Mass, Volume};

/// A typed property value carried in a property set.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum PropertyValue {
    /// Free text.
    Text(String),
    /// Numeric scalar (no unit).
    Number(f64),
    /// Boolean flag.
    Boolean(bool),
}

/// A named property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Property {
    /// Property name.
    pub name: String,
    /// Property value.
    pub value: PropertyValue,
}

/// A set of related properties, e.g. `Pset_WallCommon`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PropertySet {
    /// Property set name.
    pub name: String,
    /// Members.
    pub properties: Vec<Property>,
}

impl PropertySet {
    /// Build a property set.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: Vec::new(),
        }
    }
    /// Add a property, returning self for chaining.
    pub fn with(mut self, name: impl Into<String>, value: PropertyValue) -> Self {
        self.properties.push(Property {
            name: name.into(),
            value,
        });
        self
    }
}

/// A physical quantity extracted from (or assigned to) an element.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Quantity {
    /// Linear measure (e.g. length of a beam).
    Length(Length),
    /// Area measure (e.g. formwork).
    Area(Area),
    /// Volume measure (e.g. concrete).
    Volume(Volume),
    /// Count ("each").
    Count(Count),
    /// Mass (e.g. rebar).
    Mass(Mass),
    /// Duration.
    Duration(Duration),
}

impl Quantity {
    /// The magnitude expressed in the canonical base unit (SI base units used by
    /// [`tpt_c_units`]).
    pub fn base_value(&self) -> f64 {
        match self {
            Quantity::Length(l) => l.meters(),
            Quantity::Area(a) => a.square_meters(),
            Quantity::Volume(v) => v.cubic_meters(),
            Quantity::Count(c) => c.each(),
            Quantity::Mass(m) => m.kilograms(),
            Quantity::Duration(d) => d.hours(),
        }
    }
}

/// A named set of quantities, e.g. `BaseQuantities`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuantitySet {
    /// Quantity set name.
    pub name: String,
    /// Members.
    pub quantities: Vec<NamedQuantity>,
}

/// A quantity with a name and optional classification code.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedQuantity {
    /// Quantity name, e.g. `GrossVolume`.
    pub name: String,
    /// The quantity itself.
    pub quantity: Quantity,
}

impl QuantitySet {
    /// Build a quantity set.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            quantities: Vec::new(),
        }
    }
    /// Add a named quantity.
    pub fn with(mut self, name: impl Into<String>, quantity: Quantity) -> Self {
        self.quantities.push(NamedQuantity {
            name: name.into(),
            quantity,
        });
        self
    }
}

/// A single layer of a layered material element (wall, slab, roof).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MaterialLayer {
    /// Material name / identifier.
    pub material: String,
    /// Layer thickness.
    pub thickness: Length,
}

/// A physical, individually identifiable building element.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Element {
    /// Stable element id.
    pub id: ElementId,
    /// Human-readable name.
    pub name: String,
    /// Coarse element category (Wall, Slab, Column, ...).
    pub category: String,
    /// Optional parent storey.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storey_id: Option<String>,
    /// Layer stack for layered elements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_layers: Vec<MaterialLayer>,
    /// Attached property sets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub property_sets: Vec<PropertySet>,
    /// Attached quantity sets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quantity_sets: Vec<QuantitySet>,
    /// Optional classification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classification: Option<Classification>,
    /// Optional external id (IFC GUID, Revit id, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<ExternalId>,
}

impl Element {
    /// Build an element.
    pub fn new(id: ElementId, name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            category: category.into(),
            storey_id: None,
            material_layers: Vec::new(),
            property_sets: Vec::new(),
            quantity_sets: Vec::new(),
            classification: None,
            external_id: None,
        }
    }
    /// Set the parent storey id.
    pub fn in_storey(mut self, storey_id: impl Into<String>) -> Self {
        self.storey_id = Some(storey_id.into());
        self
    }
    /// Attach a classification.
    pub fn classified(mut self, c: Classification) -> Self {
        self.classification = Some(c);
        self
    }
    /// Attach an external id.
    pub fn with_external_id(mut self, id: ExternalId) -> Self {
        self.external_id = Some(id);
        self
    }
    /// Add a material layer.
    pub fn with_layer(mut self, material: impl Into<String>, thickness: Length) -> Self {
        self.material_layers.push(MaterialLayer {
            material: material.into(),
            thickness,
        });
        self
    }
    /// Add a property set.
    pub fn with_property_set(mut self, ps: PropertySet) -> Self {
        self.property_sets.push(ps);
        self
    }
    /// Add a quantity set.
    pub fn with_quantity_set(mut self, qs: QuantitySet) -> Self {
        self.quantity_sets.push(qs);
        self
    }
}

impl Identified for Element {
    type Id = ElementId;
    fn id(&self) -> ElementId {
        self.id
    }
}

/// A grouping of elements assembled together (e.g. a curtain wall unit).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assembly {
    /// Stable assembly id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Constituent elements.
    pub elements: Vec<ElementId>,
}

/// A spatial zone (room, space) used for area takeoffs and FM.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    /// Stable zone id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Gross area if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gross_area: Option<Area>,
}

/// A functional or discipline system grouping elements (HVAC, fire, struct).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct System {
    /// Stable system id.
    pub id: String,
    /// Name.
    pub name: String,
    /// System classification (e.g. `HVAC`, `Structural`).
    pub system_type: String,
    /// Member elements.
    pub elements: Vec<ElementId>,
}

/// A building storey / level.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Storey {
    /// Stable storey id.
    pub id: String,
    /// Name (e.g. `Level 2`).
    pub name: String,
    /// Elevation above datum.
    pub elevation: Length,
    /// Member elements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ElementId>,
}

/// A building aggregating storeys.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Building {
    /// Stable building id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Storeys.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub storeys: Vec<Storey>,
}

/// A site holding buildings and standalone elements.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Site {
    /// Stable site id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Buildings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub buildings: Vec<Building>,
    /// Elements not belonging to a building.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ElementId>,
}

/// The root project aggregate: containers plus a flat element registry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
    /// Project id.
    pub id: ProjectId,
    /// Project name.
    pub name: String,
    /// Default classification system used by the project.
    pub default_system: ClassificationSystem,
    /// Sites.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sites: Vec<Site>,
    /// All elements, indexed by id for fast lookup.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,
}

impl Project {
    /// Build an empty project.
    pub fn new(id: ProjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            default_system: ClassificationSystem::MasterFormat,
            sites: Vec::new(),
            elements: Vec::new(),
        }
    }

    /// Set the default classification system.
    pub fn with_default_system(mut self, system: ClassificationSystem) -> Self {
        self.default_system = system;
        self
    }

    /// Add an element to the registry.
    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }

    /// Look up an element by id.
    pub fn element(&self, id: ElementId) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }

    /// All elements of a given category.
    pub fn elements_of_category<'a>(
        &'a self,
        category: &'a str,
    ) -> impl Iterator<Item = &'a Element> {
        self.elements.iter().filter(move |e| e.category == category)
    }

    /// Total count of elements.
    pub fn element_count(&self) -> usize {
        self.elements.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_classification::ClassificationSystem;
    use tpt_c_ids::IdFactory;

    #[test]
    fn build_and_query_project() {
        let mut p = Project::new(ProjectId::nil(), "Demo")
            .with_default_system(ClassificationSystem::UniFormat);
        let e = Element::new(IdFactory::element(), "W1", "Wall")
            .classified(Classification::new(
                ClassificationSystem::MasterFormat,
                "03 30 00",
            ))
            .with_quantity_set(
                QuantitySet::new("BaseQuantities")
                    .with("Length", Quantity::Length(Length::from_feet(20.0))),
            );
        p.add_element(e);
        assert_eq!(p.element_count(), 1);
        assert_eq!(p.elements_of_category("Wall").count(), 1);
        let el = p.elements.first().unwrap();
        assert_eq!(
            el.quantity_sets[0].quantities[0].quantity,
            Quantity::Length(Length::from_feet(20.0))
        );
    }
}
