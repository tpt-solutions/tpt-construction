// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Phase 2 integration example: parse a sample IFC file into the neutral
//! [`tpt_c_model::Project`] and print a short summary.
//!
//! Run with `cargo run -p ifc-import -- [path.ifc]`. When no path is given the
//! bundled [`test-data/ifc/sample.ifc`](../../../test-data/ifc/sample.ifc)
//! fixture is used.

use std::process::exit;

use tpt_c_ifc::{parse, to_model};
use tpt_c_model::Project;

/// Parse `source` (IFC text) into a neutral project.
pub fn import(source: &str) -> Result<Project, tpt_c_ifc::IfcError> {
    let doc = parse(source)?;
    to_model(&doc)
}

/// Produce a one-line-per-element summary string.
pub fn summarize(project: &Project) -> String {
    let mut out = String::new();
    out.push_str(&format!("Project: {}\n", project.name));
    out.push_str(&format!("Elements: {}\n", project.element_count()));
    for element in &project.elements {
        let storey = element.storey_id.as_deref().unwrap_or("-");
        out.push_str(&format!(
            "  - [{}] {} (storey: {})\n",
            element.category, element.name, storey
        ));
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let source = if let Some(path) = args.get(1) {
        match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("failed to read {}: {}", path, e);
                exit(1);
            }
        }
    } else {
        include_str!("../../../test-data/ifc/sample.ifc").to_string()
    };

    match import(&source) {
        Ok(project) => {
            print!("{}", summarize(&project));
        }
        Err(e) => {
            eprintln!("IFC import failed: {}", e);
            exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imports_bundled_fixture() {
        let src = include_str!("../../../test-data/ifc/sample.ifc");
        let project = import(src).expect("import");
        assert_eq!(project.name, "Riverside Office Building");
        // wall, slab, column, beam = 4 elements.
        assert_eq!(project.element_count(), 4);
        let categories: Vec<&str> = project
            .elements
            .iter()
            .map(|e| e.category.as_str())
            .collect();
        assert!(categories.contains(&"Wall"));
        assert!(categories.contains(&"Column"));
        // Quantities should have been mapped from the element quantity.
        let wall = project
            .elements
            .iter()
            .find(|e| e.category == "Wall")
            .unwrap();
        assert!(wall
            .quantity_sets
            .iter()
            .any(|qs| qs.name == "BaseQuantities"));
        // Properties from the property set.
        assert!(wall
            .property_sets
            .iter()
            .any(|ps| ps.name == "Pset_WallCommon"));
    }
}
