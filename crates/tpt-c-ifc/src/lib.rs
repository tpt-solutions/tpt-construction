// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! IFC (ISO 10303-21 STEP) parser and mapper to the neutral model.
//!
//! Supports the entity families needed for takeoff and coordination:
//! `IfcProject`, `IfcSite`, `IfcBuilding`, `IfcBuildingStorey`, `IfcWall`,
//! `IfcSlab`, `IfcColumn`, `IfcBeam`, `IfcDoor`, `IfcWindow`, `IfcSpace`,
//! `IfcPropertySet` and `IfcElementQuantity`. Geometry representation parsing
//! is out of scope; quantities and property sets are extracted and attached to
//! the resulting [`tpt_c_model::Project`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tpt_c_classification::ClassificationSystem;
use tpt_c_core::ProjectId;
use tpt_c_ids::IdFactory;
use tpt_c_model::{
    Element, Project, PropertySet, PropertyValue, Quantity, QuantitySet,
};
use tpt_c_units::{Area, Count, Length, Mass, Volume};

/// Errors produced while parsing IFC.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IfcError {
    /// The STEP container markers were not found.
    #[error("not a valid STEP file (missing ISO-10303-21 markers)")]
    NotStep,
    /// A value could not be parsed.
    #[error("parse error near: {0}")]
    ParseError(String),
    /// A referenced entity id was missing.
    #[error("dangling reference #{0}")]
    DanglingRef(usize),
    /// No IfcProject was present.
    #[error("no IfcProject found in file")]
    NoProject,
}

type Result<T> = std::result::Result<T, IfcError>;

/// A parsed STEP value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StepValue {
    /// `#id` reference.
    Ref(usize),
    /// Single-quoted string.
    Str(String),
    /// Numeric literal.
    Num(f64),
    /// `.ENUMERATION.` value.
    Enum(String),
    /// `(a, b, c)` list.
    List(Vec<StepValue>),
    /// `EntityName(params)` value.
    Entity(String, Vec<StepValue>),
    /// `$` unset.
    Unset,
}

/// A single `#id = TYPE(params);` instance.
#[derive(Clone, Debug, PartialEq)]
pub struct StepInstance {
    /// Instance id.
    pub id: usize,
    /// Entity type name (e.g. `IFCWALL`).
    pub ty: String,
    /// Parameter values.
    pub params: Vec<StepValue>,
}

fn find_matching_paren(s: &str) -> Result<usize> {
    if !s.starts_with('(') {
        return Err(IfcError::ParseError("expected '('".into()));
    }
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            '\'' => {
                let mut j = i + 1;
                let b = s.as_bytes();
                while j < b.len() {
                    if b[j] == b'\'' {
                        if j + 1 < b.len() && b[j + 1] == b'\'' {
                            j += 2;
                            continue;
                        }
                        break;
                    }
                    j += 1;
                }
            }
            _ => {}
        }
    }
    Err(IfcError::ParseError("unbalanced parentheses".into()))
}

fn split_top_level(inner: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let b = inner.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'\'' => {
                i += 1;
                while i < b.len() {
                    if b[i] == b'\'' {
                        if i + 1 < b.len() && b[i + 1] == b'\'' {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            b',' if depth == 0 => {
                out.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    out.push(&inner[start..]);
    out
}

fn parse_value(s: &str) -> Result<(StepValue, &str)> {
    let s = s.trim_start();
    let b = s.as_bytes();
    if b.is_empty() {
        return Err(IfcError::ParseError("unexpected end of input".into()));
    }
    match b[0] {
        b'$' | b'*' => Ok((StepValue::Unset, &s[1..])),
        b'\'' => {
            let mut i = 1;
            let mut out = String::new();
            while i < s.len() {
                let c = b[i];
                if c == b'\'' {
                    if i + 1 < s.len() && b[i + 1] == b'\'' {
                        out.push('\'');
                        i += 2;
                        continue;
                    }
                    i += 1;
                    break;
                }
                out.push(s.as_bytes()[i] as char);
                i += 1;
            }
            Ok((StepValue::Str(out), &s[i..]))
        }
        b'#' => {
            let rest = &s[1..];
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            let id: usize = rest[..end]
                .parse()
                .map_err(|_| IfcError::ParseError("bad reference".into()))?;
            Ok((StepValue::Ref(id), &rest[end..]))
        }
        b'(' => {
            let close = find_matching_paren(s)?;
            let inner = &s[1..close];
            let segs = split_top_level(inner);
            let mut items = Vec::new();
            for seg in segs {
                let seg = seg.trim();
                if seg.is_empty() {
                    continue;
                }
                let (v, _) = parse_value(seg)?;
                items.push(v);
            }
            Ok((StepValue::List(items), &s[close + 1..]))
        }
        b'.' => {
            let rest = &s[1..];
            let end = rest.find('.').ok_or(IfcError::ParseError("bad enum".into()))?;
            let e = rest[..end].to_string();
            Ok((StepValue::Enum(e), &rest[end + 1..]))
        }
        c if c.is_ascii_alphabetic() => {
            let end = s
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(s.len());
            let name = s[..end].to_string();
            let after = s[end..].trim_start();
            if after.starts_with('(') {
                let close = find_matching_paren(after)?;
                let inner = &after[1..close];
                let segs = split_top_level(inner);
                let mut params = Vec::new();
                for seg in segs {
                    let seg = seg.trim();
                    if seg.is_empty() {
                        continue;
                    }
                    let (v, _) = parse_value(seg)?;
                    params.push(v);
                }
                Ok((StepValue::Entity(name, params), &after[close + 1..]))
            } else {
                Ok((StepValue::Entity(name, Vec::new()), after))
            }
        }
        _ => {
            let end = s
                .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E'))
                .unwrap_or(s.len());
            if end == 0 {
                return Err(IfcError::ParseError(format!("unexpected token: {}", &s[..1])));
            }
            let n: f64 = s[..end]
                .trim()
                .parse()
                .map_err(|_| IfcError::ParseError("bad number".into()))?;
            Ok((StepValue::Num(n), &s[end..]))
        }
    }
}

/// Parse the DATA section of a STEP file into instances.
pub fn parse_step(data: &str) -> Result<Vec<StepInstance>> {
    let start = data.find("DATA;").ok_or(IfcError::NotStep)?;
    let data = &data[start..];
    let mut out = Vec::new();
    let bytes = data.as_bytes();
    let mut i = 0;
    while i < data.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        let start_id = i + 1;
        let end_id = data[start_id..]
            .find(|c: char| !c.is_ascii_digit())
            .map(|p| start_id + p)
            .unwrap_or(data.len());
        let id: usize = data[start_id..end_id]
            .parse()
            .map_err(|_| IfcError::ParseError("bad instance id".into()))?;
        i = end_id;
        while i < data.len() && (bytes[i] == b' ' || bytes[i] == b'=' || bytes[i] == b'\t') {
            i += 1;
        }
        let tstart = i;
        while i < data.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
            i += 1;
        }
        let ty = data[tstart..i].to_string();
        while i < data.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= data.len() || bytes[i] != b'(' {
            return Err(IfcError::ParseError(format!("expected '(' after {}", ty)));
        }
        let slice = &data[i..];
        let (params, rest) = {
            let close = find_matching_paren(slice)?;
            let inner = &slice[1..close];
            let segs = split_top_level(inner);
            let mut items = Vec::new();
            for seg in segs {
                let seg = seg.trim();
                if seg.is_empty() {
                    continue;
                }
                let (v, _) = parse_value(seg)?;
                items.push(v);
            }
            (items, &slice[close + 1..])
        };
        out.push(StepInstance { id, ty, params });
        let consumed = slice.len() - rest.len();
        i += consumed;
        while i < data.len() && bytes[i] != b';' {
            i += 1;
        }
        i += 1;
    }
    Ok(out)
}

/// A parsed IFC document.
#[derive(Clone, Debug)]
pub struct IfcModel {
    instances: HashMap<usize, StepInstance>,
}

impl IfcModel {
    /// Parse a STEP document into a model.
    pub fn from_step(data: &str) -> Result<Self> {
        let instances = parse_step(data)?
            .into_iter()
            .map(|i| (i.id, i))
            .collect();
        Ok(Self { instances })
    }

    /// All instances of a given entity type (case-insensitive).
    pub fn entities_of_type<'a>(&'a self, ty: &'a str) -> impl Iterator<Item = &'a StepInstance> {
        let t = ty.to_uppercase();
        self.instances
            .values()
            .filter(move |i| i.ty.eq_ignore_ascii_case(&t))
    }

    /// Resolve a reference to its instance.
    pub fn instance(&self, id: usize) -> Result<&StepInstance> {
        self.instances.get(&id).ok_or(IfcError::DanglingRef(id))
    }

    fn as_str(v: &StepValue) -> Option<&str> {
        if let StepValue::Str(s) = v {
            Some(s)
        } else {
            None
        }
    }

    fn to_number(v: &StepValue) -> Option<f64> {
        match v {
            StepValue::Num(n) => Some(*n),
            StepValue::Entity(_, p) if !p.is_empty() => Self::to_number(&p[0]),
            _ => None,
        }
    }

    fn to_property_value(v: &StepValue) -> Option<PropertyValue> {
        match v {
            StepValue::Str(s) => Some(PropertyValue::Text(s.clone())),
            StepValue::Num(n) => Some(PropertyValue::Number(*n)),
            StepValue::Enum(e) => Some(PropertyValue::Text(e.clone())),
            StepValue::Entity(name, p) if !p.is_empty() => {
                let n = name.to_uppercase();
                if n.contains("BOOLEAN") {
                    let b = matches!(p[0], StepValue::Enum(ref e) if e.eq_ignore_ascii_case("true"));
                    Some(PropertyValue::Boolean(b))
                } else if n.contains("MEASURE") || n.contains("REAL") || n.contains("INTEGER") || n.contains("COUNT") {
                    Self::to_number(v).map(PropertyValue::Number)
                } else {
                    Self::to_property_value(&p[0])
                }
            }
            _ => None,
        }
    }

    fn extract_property_set(&self, inst: &StepInstance) -> Result<PropertySet> {
        let name = Self::as_str(inst.params.get(2).unwrap_or(&StepValue::Unset))
            .unwrap_or("PropertySet")
            .to_string();
        let mut ps = PropertySet::new(name);
        if let Some(StepValue::List(props)) = inst.params.get(4) {
            for p in props {
                if let StepValue::Ref(rid) = p {
                    let pinst = self.instance(*rid)?;
                    if !pinst.ty.eq_ignore_ascii_case("IfcPropertySingleValue") {
                        continue;
                    }
                    let pname = Self::as_str(pinst.params.get(0).unwrap_or(&StepValue::Unset))
                        .unwrap_or("")
                        .to_string();
                    if let Some(value) = Self::to_property_value(pinst.params.get(2).unwrap_or(&StepValue::Unset)) {
                        ps = ps.with(pname, value);
                    }
                }
            }
        }
        Ok(ps)
    }

    fn extract_quantity_set(&self, inst: &StepInstance) -> Result<QuantitySet> {
        let name = Self::as_str(inst.params.get(2).unwrap_or(&StepValue::Unset))
            .unwrap_or("QuantitySet")
            .to_string();
        let mut qs = QuantitySet::new(name);
        if let Some(StepValue::List(qs_items)) = inst.params.get(5) {
            for q in qs_items {
                if let StepValue::Ref(rid) = q {
                    let qinst = self.instance(*rid)?;
                    let qname = Self::as_str(qinst.params.get(0).unwrap_or(&StepValue::Unset))
                        .unwrap_or("")
                        .to_string();
                    let value = qinst.params.get(2).unwrap_or(&StepValue::Unset);
                    let num = Self::to_number(value);
                    let ty = qinst.ty.to_uppercase();
                    if let Some(n) = num {
                        let quantity = if ty.contains("LENGTH") {
                            Quantity::Length(Length::from_meters(n))
                        } else if ty.contains("AREA") {
                            Quantity::Area(Area::from_square_meters(n))
                        } else if ty.contains("VOLUME") {
                            Quantity::Volume(Volume::from_cubic_meters(n))
                        } else if ty.contains("COUNT") {
                            Quantity::Count(Count::from_each(n))
                        } else if ty.contains("WEIGHT") {
                            Quantity::Mass(Mass::from_kilograms(n))
                        } else {
                            continue;
                        };
                        qs = qs.with(qname, quantity);
                    }
                }
            }
        }
        Ok(qs)
    }

    /// Build a neutral [`Project`] from the parsed IFC.
    pub fn to_model(&self) -> Result<Project> {
        let project_inst = self
            .entities_of_type("IfcProject")
            .next()
            .ok_or(IfcError::NoProject)?;
        let project_name = Self::as_str(project_inst.params.get(2).unwrap_or(&StepValue::Unset))
            .unwrap_or("IFC Project")
            .to_string();

        let element_types = [
            "IfcWall",
            "IfcWallStandardCase",
            "IfcSlab",
            "IfcColumn",
            "IfcBeam",
            "IfcDoor",
            "IfcWindow",
            "IfcSpace",
        ];

        let mut element_props: HashMap<usize, Vec<usize>> = HashMap::new();
        for rel in self.entities_of_type("IfcRelDefinesByProperties") {
            let related = rel.params.get(4);
            let relating = rel.params.get(5);
            let mut def_ids = Vec::new();
            if let Some(StepValue::Ref(r)) = relating {
                def_ids.push(*r);
            } else if let Some(StepValue::List(items)) = relating {
                for it in items {
                    if let StepValue::Ref(r) = it {
                        def_ids.push(*r);
                    }
                }
            }
            let mut obj_ids = Vec::new();
            if let Some(StepValue::List(items)) = related {
                for it in items {
                    if let StepValue::Ref(r) = it {
                        obj_ids.push(*r);
                    }
                }
            }
            for o in obj_ids {
                element_props.entry(o).or_default().extend(def_ids.iter().copied());
            }
        }

        let mut project = Project::new(ProjectId::nil(), project_name)
            .with_default_system(ClassificationSystem::MasterFormat);

        for ty in element_types {
            for inst in self.entities_of_type(ty) {
                let name = Self::as_str(inst.params.get(2).unwrap_or(&StepValue::Unset))
                    .unwrap_or(ty)
                    .to_string();
                let eid = IdFactory::deterministic_element(&format!("#{}", inst.id));
                let mut element =
                    Element::new(eid, name, inst.ty.clone()).with_external_id(
                        tpt_c_ids::ExternalId::IfcGuid(
                            Self::as_str(inst.params.get(0).unwrap_or(&StepValue::Unset))
                                .unwrap_or("")
                                .to_string(),
                        ),
                    );
                if let Some(defs) = element_props.get(&inst.id) {
                    for d in defs {
                        let dinst = self.instance(*d)?;
                        if dinst.ty.eq_ignore_ascii_case("IfcPropertySet") {
                            element = element.with_property_set(self.extract_property_set(dinst)?);
                        } else if dinst.ty.eq_ignore_ascii_case("IfcElementQuantity") {
                            element = element.with_quantity_set(self.extract_quantity_set(dinst)?);
                        }
                    }
                }
                project.add_element(element);
            }
        }

        Ok(project)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('ViewDefinition'),'2;1');
FILE_NAME('sample.ifc','2026-01-01T00:00:00',('TPT'),(''),'','','');
FILE_SCHEMA(('IFC4'));
ENDSEC;
DATA;
#1=IFCPROJECT('0YvctvNInA9g7gQ9yzQzvZ',$,'Demo Project',$,$,$,$,$,$);
#2=IFCPROPERTYSET('2O0MLfx3BDgvhRaydMkJh7',$,'Pset_WallCommon',$,(#5));
#5=IFCPROPERTYSINGLEVALUE('IsExternal',$,IFCBOOLEAN(.TRUE.),$);
#6=IFCELEMENTQUANTITY('3O0MLfx3BDgvhRaydMkJh8',$,'BaseQuantities',$,$,(#7));
#7=IFCQUANTITYLENGTH('Length',$,IFCLENGTHMEASURE(10.));
#10=IFCWALLSTANDARDCASE('1O0MLfx3BDgvhRaydMkJh0',$,'West Wall',$,$,$,$,$);
#11=IFCRELDEFINESBYPROPERTIES('4O0MLfx3BDgvhRaydMkJh1',$,$,$,(#10),#2);
#12=IFCRELDEFINESBYPROPERTIES('5O0MLfx3BDgvhRaydMkJh2',$,$,$,(#10),#6);
ENDSEC;
END-ISO-10303-21;
"#;

    #[test]
    fn parse_and_map() {
        let model = IfcModel::from_step(SAMPLE).unwrap();
        let project = model.to_model().unwrap();
        assert_eq!(project.name, "Demo Project");
        assert_eq!(project.element_count(), 1);
        let wall = &project.elements[0];
        assert_eq!(wall.name, "West Wall");
        assert!(wall.property_sets.iter().any(|p| p.name == "Pset_WallCommon"));
        let ps = wall.property_sets.iter().find(|p| p.name == "Pset_WallCommon").unwrap();
        let ext = ps.properties.iter().find(|p| p.name == "IsExternal").unwrap();
        assert_eq!(ext.value, PropertyValue::Boolean(true));
        let qs = wall.quantity_sets.iter().find(|q| q.name == "BaseQuantities").unwrap();
        assert_eq!(qs.quantities[0].quantity, Quantity::Length(Length::from_meters(10.0)));
    }
}
