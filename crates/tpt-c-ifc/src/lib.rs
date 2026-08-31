// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! IFC (ISO 10303-21 / STEP) parser and mapper to the neutral model.
//!
//! Supports the building elements needed for quantity takeoff and coordination:
//! `IfcProject`, `IfcSite`, `IfcBuilding`, `IfcBuildingStorey`, `IfcWall`,
//! `IfcSlab`, `IfcColumn`, `IfcBeam`, `IfcDoor`, `IfcWindow`, `IfcSpace`, plus
//! property sets and element quantities. Geometry (B-rep / tessellation) is not
//! decoded; instead the parser extracts the semantic structure, classifications,
//! properties, and quantities and maps them onto [`tpt_c_model`].

use std::collections::HashMap;

use thiserror::Error;
use tpt_c_core::ProjectId;
use tpt_c_ids::{ExternalId, IdFactory};
use tpt_c_model::{
    Element, PropertySet, PropertyValue, Project, Quantity, QuantitySet,
};
use tpt_c_units::{Area, Count, Length, Mass, Volume};

/// Errors raised while parsing or mapping an IFC document.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum IfcError {
    /// The file was not a valid ISO-10303-21 document.
    #[error("invalid IFC header: {0}")]
    InvalidHeader(String),
    /// The data section was malformed.
    #[error("parse error near offset {0}: {1}")]
    Parse(usize, String),
    /// A referenced entity id did not exist.
    #[error("dangling reference #{0}")]
    DanglingRef(usize),
    /// A required entity (IfcProject) was missing.
    #[error("missing IfcProject in document")]
    MissingProject,
}

/// A value in the STEP parameter grammar.
#[derive(Debug, Clone, PartialEq)]
pub enum StepValue {
    /// An entity reference, e.g. `#12`.
    Ref(usize),
    /// A string literal (quotes already stripped, `''` unescaped).
    Str(String),
    /// A real number.
    Real(f64),
    /// An integer.
    Int(i64),
    /// An enumeration literal (dots stripped), e.g. `WALL`.
    Enum(String),
    /// A list of values.
    List(Vec<StepValue>),
    /// `$` — attribute not assigned.
    Unspecified,
    /// `*` — attribute redeclared/inherited.
    Star,
}

/// A single STEP entity instance: `#id = TYPE(...)`.
#[derive(Debug, Clone, PartialEq)]
pub struct StepEntity {
    /// The instance id (`#id`).
    pub id: usize,
    /// The entity type name (e.g. `IFCWALL`).
    pub ty: String,
    /// Positional parameters.
    pub params: Vec<StepValue>,
}

impl StepEntity {
    /// String at `i`, if present and a string.
    pub fn str_at(&self, i: usize) -> Option<&str> {
        match self.params.get(i)? {
            StepValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }
    /// Real at `i`, if present and numeric (int or real).
    pub fn real_at(&self, i: usize) -> Option<f64> {
        match self.params.get(i)? {
            StepValue::Real(r) => Some(*r),
            StepValue::Int(n) => Some(*n as f64),
            _ => None,
        }
    }
    /// Integer at `i`.
    pub fn int_at(&self, i: usize) -> Option<i64> {
        match self.params.get(i)? {
            StepValue::Int(n) => Some(*n),
            _ => None,
        }
    }
    /// Entity reference at `i`.
    pub fn ref_at(&self, i: usize) -> Option<usize> {
        match self.params.get(i)? {
            StepValue::Ref(r) => Some(*r),
            _ => None,
        }
    }
    /// List at `i`.
    pub fn list_at(&self, i: usize) -> Option<&[StepValue]> {
        match self.params.get(i)? {
            StepValue::List(l) => Some(l.as_slice()),
            _ => None,
        }
    }
}

/// A parsed STEP document.
#[derive(Debug, Clone, Default)]
pub struct StepDoc {
    /// All entities keyed by instance id.
    pub entities: HashMap<usize, StepEntity>,
}

impl StepDoc {
    /// Look up an entity by id.
    pub fn get(&self, id: usize) -> Result<&StepEntity, IfcError> {
        self.entities.get(&id).ok_or(IfcError::DanglingRef(id))
    }

    /// All entities of a given type (case-insensitive).
    pub fn by_type(&self, ty: &str) -> Vec<&StepEntity> {
        let ty = ty.to_ascii_uppercase();
        self.entities.values().filter(|e| e.ty == ty).collect()
    }

    /// All entities whose type starts with `prefix` (case-insensitive).
    pub fn by_type_prefix(&self, prefix: &str) -> Vec<&StepEntity> {
        let prefix = prefix.to_ascii_uppercase();
        self.entities
            .values()
            .filter(|e| e.ty.starts_with(&prefix))
            .collect()
    }
}

/// Parse an entire ISO-10303-21 document.
pub fn parse(input: &str) -> Result<StepDoc, IfcError> {
    let bytes: Vec<char> = input.chars().collect();
    let mut p = Parser { s: &bytes, pos: 0 };
    p.skip_ws();
    p.expect_keyword("ISO-10303-21")?;
    p.skip_ws();
    p.expect_char(';')?;
    p.skip_ws();
    p.expect_keyword("HEADER")?;
    p.skip_ws();
    p.expect_char(';')?;
    p.skip_ws();
    // Consume header records until ENDSEC.
    loop {
        p.skip_ws();
        if p.peek_keyword("ENDSEC") {
            p.advance(6);
            p.skip_ws();
            p.expect_char(';')?;
            break;
        }
        p.skip_record()?;
        p.skip_ws();
    }
    p.skip_ws();
    p.expect_keyword("DATA")?;
    p.skip_ws();
    p.expect_char(';')?;
    p.skip_ws();

    let mut doc = StepDoc::default();
    loop {
        p.skip_ws();
        if p.peek_keyword("ENDSEC") {
            p.advance(6);
            p.skip_ws();
            p.expect_char(';')?;
            break;
        }
        let entity = p.parse_entity_instance()?;
        doc.entities.insert(entity.id, entity);
        p.skip_ws();
    }
    p.expect_keyword("END-ISO-10303-21")?;
    p.skip_ws();
    p.expect_char(';')?;
    Ok(doc)
}

struct Parser<'a> {
    s: &'a [char],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c.is_whitespace() {
                self.pos += 1;
            } else if c == '/' && self.peek_char(1) == Some('*') {
                self.pos += 2;
                while self.pos < self.s.len() && !(self.s[self.pos] == '*' && self.peek_char(1) == Some('/')) {
                    self.pos += 1;
                }
                self.pos += 2;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self, ahead: usize) -> Option<char> {
        self.s.get(self.pos + ahead).copied()
    }

    fn peek_keyword(&self, kw: &str) -> bool {
        let chars: Vec<char> = kw.chars().collect();
        if self.pos + chars.len() > self.s.len() {
            return false;
        }
        self.s[self.pos..self.pos + chars.len()].iter().eq(chars.iter())
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    fn expect_char(&mut self, c: char) -> Result<(), IfcError> {
        self.skip_ws();
        if self.peek_char(0) != Some(c) {
            return Err(IfcError::Parse(self.pos, format!("expected '{c}'")));
        }
        self.pos += 1;
        Ok(())
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), IfcError> {
        if !self.peek_keyword(kw) {
            return Err(IfcError::Parse(self.pos, format!("expected keyword '{kw}'")));
        }
        self.advance(kw.chars().count());
        Ok(())
    }

    fn skip_record(&mut self) -> Result<(), IfcError> {
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c == ';' {
                self.pos += 1;
                return Ok(());
            }
            if c == '(' {
                self.skip_balanced('(', ')')?;
            } else {
                self.pos += 1;
            }
        }
        Err(IfcError::Parse(self.pos, "unexpected EOF in header".into()))
    }

    fn skip_balanced(&mut self, open: char, close: char) -> Result<(), IfcError> {
        let mut depth = 0;
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c == '\'' {
                self.skip_string()?;
                continue;
            }
            if c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                self.pos += 1;
                if depth == 0 {
                    return Ok(());
                }
            }
            self.pos += 1;
        }
        Err(IfcError::Parse(self.pos, "unbalanced delimiter".into()))
    }

    fn skip_string(&mut self) -> Result<(), IfcError> {
        self.pos += 1;
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c == '\'' && self.peek_char(1) == Some('\'') {
                self.pos += 2;
                continue;
            }
            if c == '\'' {
                self.pos += 1;
                return Ok(());
            }
            self.pos += 1;
        }
        Err(IfcError::Parse(self.pos, "unterminated string".into()))
    }

    fn parse_entity_instance(&mut self) -> Result<StepEntity, IfcError> {
        self.skip_ws();
        if self.peek_char(0) != Some('#') {
            return Err(IfcError::Parse(self.pos, "expected '#id='".into()));
        }
        self.pos += 1;
        let id = self.parse_uint()?;
        self.skip_ws();
        self.expect_char('=')?;
        self.skip_ws();
        let ty = self.parse_identifier()?;
        self.skip_ws();
        self.expect_char('(')?;
        let params = self.parse_value_list(')')?;
        self.skip_ws();
        self.expect_char(';')?;
        Ok(StepEntity { id, ty, params })
    }

    fn parse_uint(&mut self) -> Result<usize, IfcError> {
        let start = self.pos;
        while self.pos < self.s.len() && self.s[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s: String = self.s[start..self.pos].iter().collect();
        s.parse::<usize>()
            .map_err(|_| IfcError::Parse(start, "invalid integer id".into()))
    }

    fn parse_identifier(&mut self) -> Result<String, IfcError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c.is_ascii_alphanumeric() || c == '_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(IfcError::Parse(self.pos, "expected identifier".into()));
        }
        Ok(self.s[start..self.pos].iter().collect())
    }

    fn parse_value_list(&mut self, close: char) -> Result<Vec<StepValue>, IfcError> {
        let mut out = Vec::new();
        self.skip_ws();
        if self.peek_char(0) == Some(close) {
            self.pos += 1;
            return Ok(out);
        }
        loop {
            let v = self.parse_value()?;
            out.push(v);
            self.skip_ws();
            match self.peek_char(0) {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some(c) if c == close => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(IfcError::Parse(self.pos, "expected ',' or ')'".into())),
            }
        }
        Ok(out)
    }

    fn parse_value(&mut self) -> Result<StepValue, IfcError> {
        self.skip_ws();
        let c = match self.peek_char(0) {
            Some(c) => c,
            None => return Err(IfcError::Parse(self.pos, "unexpected EOF".into())),
        };
        match c {
            '$' => {
                self.pos += 1;
                Ok(StepValue::Unspecified)
            }
            '*' => {
                self.pos += 1;
                Ok(StepValue::Star)
            }
            '#' => {
                self.pos += 1;
                let id = self.parse_uint()?;
                Ok(StepValue::Ref(id))
            }
            '\'' => {
                let s = self.parse_string()?;
                Ok(StepValue::Str(s))
            }
            '(' => {
                self.pos += 1;
                let list = self.parse_value_list(')')?;
                Ok(StepValue::List(list))
            }
            '.' => {
                self.pos += 1;
                let start = self.pos;
                while self.pos < self.s.len() && self.s[self.pos] != '.' {
                    self.pos += 1;
                }
                let s: String = self.s[start..self.pos].iter().collect();
                if self.peek_char(0) != Some('.') {
                    return Err(IfcError::Parse(self.pos, "unterminated enum".into()));
                }
                self.pos += 1;
                Ok(StepValue::Enum(s))
            }
            '-' | '+' | '0'..='9' => self.parse_number(),
            _ => {
                let id = self.parse_identifier()?;
                self.skip_ws();
                if self.peek_char(0) == Some('(') {
                    self.pos += 1;
                    let inner = self.parse_value_list(')')?;
                    if inner.len() == 1 {
                        return Ok(inner.into_iter().next().unwrap());
                    }
                    return Ok(StepValue::List(inner));
                }
                Ok(StepValue::Enum(id))
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, IfcError> {
        self.pos += 1;
        let mut out = String::new();
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c == '\'' && self.peek_char(1) == Some('\'') {
                out.push('\'');
                self.pos += 2;
                continue;
            }
            if c == '\'' {
                self.pos += 1;
                return Ok(out);
            }
            out.push(c);
            self.pos += 1;
        }
        Err(IfcError::Parse(self.pos, "unterminated string".into()))
    }

    fn parse_number(&mut self) -> Result<StepValue, IfcError> {
        let start = self.pos;
        if self.peek_char(0) == Some('-') || self.peek_char(0) == Some('+') {
            self.pos += 1;
        }
        let mut is_real = false;
        while self.pos < self.s.len() {
            let c = self.s[self.pos];
            if c.is_ascii_digit() {
                self.pos += 1;
            } else if c == '.' {
                is_real = true;
                self.pos += 1;
            } else if c == 'e' || c == 'E' {
                is_real = true;
                self.pos += 1;
                if self.peek_char(0) == Some('-') || self.peek_char(0) == Some('+') {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
        let s: String = self.s[start..self.pos].iter().collect();
        if is_real {
            s.parse::<f64>()
                .map(StepValue::Real)
                .map_err(|_| IfcError::Parse(start, "invalid real".into()))
        } else {
            s.parse::<i64>()
                .map(StepValue::Int)
                .map_err(|_| IfcError::Parse(start, "invalid integer".into()))
        }
    }
}

/// Maps a STEP entity type to a neutral model category.
fn category_for_type(ty: &str) -> Option<&'static str> {
    match ty.to_ascii_uppercase().as_str() {
        "IFCWALL" | "IFCWALLSTANDARDCASE" => Some("Wall"),
        "IFCSLAB" => Some("Slab"),
        "IFCCOLUMN" => Some("Column"),
        "IFCBEAM" | "IFCBEAMSTANDARDCASE" => Some("Beam"),
        "IFCDOOR" => Some("Door"),
        "IFCWINDOW" => Some("Window"),
        "IFCSPACE" => Some("Space"),
        _ => None,
    }
}

/// Convert a parsed STEP document into a neutral [`Project`].
pub fn to_model(doc: &StepDoc) -> Result<Project, IfcError> {
    let project_entity = doc
        .by_type("IFCPROJECT")
        .into_iter()
        .next()
        .ok_or(IfcError::MissingProject)?;

    let project_name = project_entity
        .str_at(2)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "IFC Project".to_string());
    let project_guid = project_entity.str_at(0).unwrap_or("").to_string();

    let project_id = ProjectId::from_uuid(IdFactory::deterministic(&project_guid));
    let mut project = Project::new(project_id, project_name);

    for entity in doc.by_type_prefix("IFC") {
        let Some(category) = category_for_type(&entity.ty) else {
            continue;
        };
        let guid = entity.str_at(0).unwrap_or("").to_string();
        if guid.is_empty() {
            continue;
        }
        let id = IdFactory::deterministic_element(&guid);
        let name = entity
            .str_at(2)
            .filter(|s| !s.is_empty())
            .unwrap_or(category)
            .to_string();

        let mut element = Element::new(id, name, category)
            .with_external_id(ExternalId::IfcGuid(guid.clone()));

        if let Some(storey) = contained_structure_name(doc, entity.id) {
            element = element.in_storey(storey);
        }

        for (ps_name, props) in property_sets_for(doc, entity.id)? {
            let mut ps = PropertySet::new(ps_name);
            for (k, v) in props {
                ps = ps.with(k, v);
            }
            element = element.with_property_set(ps);
        }
        for (qs_name, quants) in quantity_sets_for(doc, entity.id)? {
            let mut qs = QuantitySet::new(qs_name);
            for (k, q) in quants {
                qs = qs.with(k, q);
            }
            element = element.with_quantity_set(qs);
        }

        project.add_element(element);
    }

    Ok(project)
}

/// For an element, find the name of the spatial structure containing it.
fn contained_structure_name(doc: &StepDoc, element_id: usize) -> Option<String> {
    for rel in doc.by_type("IFCRELCONTAINEDINSPATIALSTRUCTURE") {
        // IfcRoot(0..3): GlobalId, OwnerHistory, Name, Description
        // then RelatedElements(4), RelatingStructure(5)
        if let Some(list) = rel.list_at(4) {
            let contains = list.iter().any(|v| matches!(v, StepValue::Ref(r) if *r == element_id));
            if contains {
                if let Some(struct_id) = rel.ref_at(5) {
                    if let Ok(s) = doc.get(struct_id) {
                        if let Some(n) = s.str_at(2).filter(|n| !n.is_empty()) {
                            return Some(n.to_string());
                        }
                        return Some(format!("#{}", struct_id));
                    }
                }
            }
        }
    }
    None
}

/// Collect (property-set-name, [(prop-name, value)]) for an element.
fn property_sets_for(
    doc: &StepDoc,
    element_id: usize,
) -> Result<Vec<(String, Vec<(String, PropertyValue)>)>, IfcError> {
    let mut out = Vec::new();
    for rel in doc.by_type("IFCRELDEFINESBYPROPERTIES") {
        // RelatedObjects(4), RelatingPropertyDefinition(5)
        let related: Vec<usize> = rel
            .list_at(4)
            .map(|l| {
                l.iter()
                    .filter_map(|v| match v {
                        StepValue::Ref(r) => Some(*r),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !related.contains(&element_id) {
            continue;
        }
        let Some(ps_id) = rel.ref_at(5) else { continue };
        let ps = doc.get(ps_id)?;
        if ps.ty != "IFCPROPERTYSET" {
            continue;
        }
        let name = ps.str_at(2).unwrap_or("PropertySet").to_string();
        let mut props = Vec::new();
        if let Some(list) = ps.list_at(4) {
            for v in list {
                if let StepValue::Ref(pid) = v {
                    if let Ok(p) = doc.get(*pid) {
                        if p.ty == "IFCPROPERTYSINGLEVALUE" {
                            let pname = p.str_at(0).unwrap_or("").to_string();
                            if let Some(value) = single_value(&p.params) {
                                props.push((pname, value));
                            }
                        }
                    }
                }
            }
        }
        out.push((name, props));
    }
    Ok(out)
}

/// Extract a single property's value from IfcPropertySingleValue params.
/// (GlobalId, OwnerHistory, Name, Description, NominalValue, Unit)
fn single_value(params: &[StepValue]) -> Option<PropertyValue> {
    let v = params.get(4)?;
    match v {
        StepValue::Str(s) => Some(PropertyValue::Text(s.clone())),
        StepValue::Real(r) => Some(PropertyValue::Number(*r)),
        StepValue::Int(n) => Some(PropertyValue::Number(*n as f64)),
        StepValue::Enum(e) => Some(PropertyValue::Text(e.clone())),
        _ => None,
    }
}

/// Collect (quantity-set-name, [(q-name, quantity)]) for an element.
fn quantity_sets_for(
    doc: &StepDoc,
    element_id: usize,
) -> Result<Vec<(String, Vec<(String, Quantity)>)>, IfcError> {
    let mut out = Vec::new();
    for rel in doc.by_type("IFCRELDEFINESBYPROPERTIES") {
        let related: Vec<usize> = rel
            .list_at(4)
            .map(|l| {
                l.iter()
                    .filter_map(|v| match v {
                        StepValue::Ref(r) => Some(*r),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !related.contains(&element_id) {
            continue;
        }
        let Some(qs_id) = rel.ref_at(5) else { continue };
        let qs = doc.get(qs_id)?;
        if qs.ty != "IFCELEMENTQUANTITY" {
            continue;
        }
        let name = qs.str_at(2).unwrap_or("Quantities").to_string();
        let mut quants = Vec::new();
        for param in &qs.params {
            if let StepValue::List(list) = param {
                for v in list {
                    if let StepValue::Ref(qid) = v {
                        if let Ok(q) = doc.get(*qid) {
                            if let Some((qname, q)) = extract_quantity(q) {
                                quants.push((qname, q));
                            }
                        }
                    }
                }
            }
        }
        if !quants.is_empty() {
            out.push((name, quants));
        }
    }
    Ok(out)
}

/// Extract a single quantity from an IfcQuantity* entity.
/// (GlobalId, OwnerHistory, Name, Description, Value)
fn extract_quantity(q: &StepEntity) -> Option<(String, Quantity)> {
    let name = q.str_at(2).unwrap_or("").to_string();
    let value = match q.ty.as_str() {
        "IFCQUANTITYLENGTH" => q.real_at(4).map(|v| Quantity::Length(Length::from_meters(v))),
        "IFCQUANTITYAREA" => q.real_at(4).map(|v| Quantity::Area(Area::from_square_meters(v))),
        "IFCQUANTITYVOLUME" => q.real_at(4).map(|v| Quantity::Volume(Volume::from_cubic_meters(v))),
        "IFCQUANTITYCOUNT" => q.real_at(4).map(|v| Quantity::Count(Count::from_each(v))),
        "IFCQUANTITYWEIGHT" => q.real_at(4).map(|v| Quantity::Mass(Mass::from_kilograms(v))),
        _ => None,
    }?;
    Some((name, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('ViewDefinition [CoordinationView]'),'2;1');
FILE_NAME('sample.ifc','2026-01-01T00:00:00',('TPT'),('TPT'),'','','');
FILE_SCHEMA(('IFC2X3'));
ENDSEC;
DATA;
#1=IFCPROJECT('0YcC700kj2vxrGBVxoQqhl',$,'Demo Project',$,$,$,$,$,$);
#2=IFCSITE('3O0MLfx3BDgvhRaydMkJh6',$,'Site',$,$,$,$,$,$,$,$,$,$,$);
#3=IFCBUILDING('1a2b3c4d5e6f7g8h9i0j',$,'Building',$,$,$,$,$,$,$);
#10=IFCBUILDINGSTOREY('storey-1',$,'Level 1',$,$,$,$,$,100.);
#20=IFCWALL('wall-1',$,'East Wall',$,$,$,$,$,$,$);
#21=IFCSLAB('slab-1',$,'Floor Slab',$,$,$,$,$,$,$);
#30=IFCPROPERTYSET('ps-1',$,'Pset_WallCommon',$,(#31));
#31=IFCPROPERTYSINGLEVALUE('LoadBearing',$,'YES',$,$);
#40=IFCELEMENTQUANTITY('q-1',$,'BaseQuantities',$,(#41,#42));
#41=IFCQUANTITYLENGTH('Length',$,$,$,20.);
#42=IFCQUANTITYVOLUME('GrossVolume',$,$,$,5.);
#50=IFCRELAGGREGATES('agg-1',$,#1,(#2));
#51=IFCRELAGGREGATES('agg-2',$,#2,(#3));
#52=IFCRELAGGREGATES('agg-3',$,#3,(#10));
#60=IFCRELCONTAINEDINSPATIALSTRUCTURE('rel-1',$,(#20,#21),$,#10);
#70=IFCRELDEFINESBYPROPERTIES('relp-1',$,(#20),$,#30);
#71=IFCRELDEFINESBYPROPERTIES('relp-2',$,(#20,#21),$,#40);
ENDSEC;
END-ISO-10303-21;
"#;

    #[test]
    fn parse_and_map() {
        let doc = parse(SAMPLE).expect("parse");
        assert_eq!(doc.by_type("IFCWALL").len(), 1);
        let project = to_model(&doc).expect("map");
        assert_eq!(project.name, "Demo Project");
        assert_eq!(project.element_count(), 2);
        let wall = project.elements.iter().find(|e| e.category == "Wall").unwrap();
        assert_eq!(wall.storey_id.as_deref(), Some("Level 1"));
        assert!(wall.property_sets.iter().any(|ps| ps.name == "Pset_WallCommon"));
        let qs = wall.quantity_sets.first().unwrap();
        assert_eq!(qs.name, "BaseQuantities");
        assert_eq!(qs.quantities.len(), 2);
        let slab = project.elements.iter().find(|e| e.category == "Slab").unwrap();
        assert!(slab.quantity_sets.iter().any(|qs| qs.name == "BaseQuantities"));
    }

    #[test]
    fn string_escaping_and_reals() {
        let src = "ISO-10303-21;\nHEADER;ENDSEC;\nDATA;\n#1=IFCTEST('it''s a wall',$,12.5,$,(#2,#3));\nENDSEC;\nEND-ISO-10303-21;\n";
        let doc = parse(src).unwrap();
        let e = doc.get(1).unwrap();
        assert_eq!(e.str_at(0), Some("it's a wall"));
        assert_eq!(e.real_at(2), Some(12.5));
        assert_eq!(e.list_at(4).unwrap().len(), 2);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse("not a step file").is_err());
    }
}
