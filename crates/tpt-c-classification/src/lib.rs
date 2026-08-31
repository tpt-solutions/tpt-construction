// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construction classification systems and project-level mapping.
//!
//! Provides first-class support for the major classification taxonomies used
//! in construction (MasterFormat, UniFormat, OmniClass, Uniclass) and lets a
//! project define its own custom classification codes that still map back to a
//! recognized system.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A supported classification taxonomy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSystem {
    /// CSI MasterFormat (specifications / cost codes).
    MasterFormat,
    /// ASTM UniFormat (functional/elemental breakdown).
    UniFormat,
    /// OmniClass (integrated classification).
    OmniClass,
    /// Uniclass (UK taxonomy).
    Uniclass,
    /// A project-specific scheme.
    Custom,
}

/// A single classification code within a system.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Classification {
    /// The taxonomy this code belongs to.
    pub system: ClassificationSystem,
    /// The code itself, e.g. `03 30 00`.
    pub code: String,
    /// Human-readable title when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

impl Classification {
    /// Build a classification from parts.
    pub fn new(system: ClassificationSystem, code: impl Into<String>) -> Self {
        Self {
            system,
            code: code.into(),
            title: None,
        }
    }
    /// Attach a title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Canonical MasterFormat divisions (00–49) with titles.
pub mod masterformat {
    /// (code, title) pairs for the standard MasterFormat divisions.
    pub const DIVISIONS: &[(&str, &str)] = &[
        ("00", "Procurement and Contracting Requirements"),
        ("01", "General Requirements"),
        ("02", "Existing Conditions"),
        ("03", "Concrete"),
        ("04", "Masonry"),
        ("05", "Metals"),
        ("06", "Wood, Plastics, and Composites"),
        ("07", "Thermal and Moisture Protection"),
        ("08", "Openings"),
        ("09", "Finishes"),
        ("10", "Specialties"),
        ("11", "Equipment"),
        ("12", "Furnishings"),
        ("13", "Special Construction"),
        ("14", "Conveying Equipment"),
        ("21", "Fire Suppression"),
        ("22", "Plumbing"),
        ("23", "Heating, Ventilating, and Air Conditioning"),
        ("25", "Integrated Automation"),
        ("26", "Electrical"),
        ("27", "Communications"),
        ("28", "Electronic Safety and Security"),
        ("31", "Earthwork"),
        ("32", "Exterior Improvements"),
        ("33", "Utilities"),
        ("34", "Transportation"),
        ("35", "Waterway and Marine Construction"),
        ("40", "Process Integration"),
        ("41", "Material Processing and Handling Equipment"),
        ("42", "Material, Component, and Equipment Storage and Support"),
        ("43", "Process Equipment"),
        ("44", "Pollution Control Equipment"),
        ("45", "Industry-Specific Manufacturing Equipment"),
        ("46", "Water and Wastewater Equipment"),
        ("48", "Electrical Power Generation"),
        ("49", "Electrical Transmission and Distribution"),
    ];

    /// Look up a division title by its two-digit code.
    pub fn title_for(code: &str) -> Option<&'static str> {
        DIVISIONS
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, t)| *t)
    }
}

/// Canonical UniFormat major groups (A–H).
pub mod uniformat {
    /// (code, title) pairs for UniFormat major groups.
    pub const GROUPS: &[(&str, &str)] = &[
        ("A", "Substructure"),
        ("B", "Shell"),
        ("C", "Interiors"),
        ("D", "Services"),
        ("E", "Equipment and Furnishings"),
        ("F", "Special Construction and Demolition"),
        ("G", "Building Sitework"),
        ("H", "Infrastructure"),
    ];

    /// Look up a group title by its letter.
    pub fn title_for(code: &str) -> Option<&'static str> {
        GROUPS.iter().find(|(c, _)| *c == code).map(|(_, t)| *t)
    }
}

/// Maps a project's own category labels to an external classification.
///
/// Example: a project may call a category `foundation-slab` and map it to
/// MasterFormat `03 30 00` (Cast-in-Place Concrete).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectClassificationMap {
    forward: HashMap<String, Classification>,
    reverse: HashMap<(ClassificationSystem, String), String>,
}

impl ProjectClassificationMap {
    /// Create an empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a mapping from a project category to a classification.
    pub fn insert(&mut self, category: impl Into<String>, classification: Classification) {
        let category = category.into();
        self.reverse.insert(
            (classification.system, classification.code.clone()),
            category.clone(),
        );
        self.forward.insert(category, classification);
    }

    /// Resolve a project category to its classification.
    pub fn classify(&self, category: &str) -> Option<&Classification> {
        self.forward.get(category)
    }

    /// Resolve a classification back to the originating project category.
    pub fn category_for(&self, system: ClassificationSystem, code: &str) -> Option<&String> {
        self.reverse.get(&(system, code.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masterformat_lookup() {
        assert_eq!(masterformat::title_for("03"), Some("Concrete"));
        assert_eq!(masterformat::title_for("99"), None);
    }

    #[test]
    fn uniformat_lookup() {
        assert_eq!(uniformat::title_for("A"), Some("Substructure"));
    }

    #[test]
    fn project_map_bidirectional() {
        let mut m = ProjectClassificationMap::new();
        m.insert(
            "foundation-slab",
            Classification::new(ClassificationSystem::MasterFormat, "03 30 00")
                .with_title("Cast-in-Place Concrete"),
        );
        let c = m.classify("foundation-slab").unwrap();
        assert_eq!(c.code, "03 30 00");
        assert_eq!(
            m.category_for(ClassificationSystem::MasterFormat, "03 30 00"),
            Some(&"foundation-slab".to_string())
        );
    }
}
