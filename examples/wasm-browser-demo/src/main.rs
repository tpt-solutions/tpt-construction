// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Phase 9 integration example: consume `tpt-c-wasm` from a Rust binary that
//! simulates a browser environment. Demonstrates IFC parsing, quantity takeoff,
//! and model export via the WASM-facing API.
//!
//! Run with:
//! ```bash
//! cargo run -p wasm-browser-demo -- [path.ifc]
//! ```

use std::process::exit;

use tpt_c_ifc::{parse, to_model};
use tpt_c_model::Project;
use tpt_c_quantities::{MeasuredQuantity, TakeoffEngine};
use tpt_c_wasm::WasmResult;

/// Parse IFC text into a neutral project model.
fn import_ifc(source: &str) -> Result<Project, String> {
    let doc = parse(source).map_err(|e| e.to_string())?;
    to_model(&doc).map_err(|e| e.to_string())
}

/// Run quantity takeoff on a project.
fn run_takeoff(project: &Project) -> Result<tpt_c_quantities::TakeoffResult, String> {
    let engine = TakeoffEngine::new();
    Ok(engine.run(project))
}

/// Format a measured quantity for display.
fn fmt_quantity(q: &MeasuredQuantity) -> String {
    match q {
        MeasuredQuantity::Count(c) => format!("{} each", c.each()),
        MeasuredQuantity::Length(l) => format!("{:.2} m", l.meters()),
        MeasuredQuantity::Area(a) => format!("{:.2} m²", a.square_meters()),
        MeasuredQuantity::Volume(v) => format!("{:.3} m³", v.cubic_meters()),
        MeasuredQuantity::Mass(m) => format!("{:.2} kg", m.kilograms()),
    }
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

    let project = match import_ifc(&source) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("IFC import failed: {}", e);
            exit(1);
        }
    };

    println!("Project imported: {}", project.name);
    println!("Elements: {}", project.element_count());

    let takeoff = match run_takeoff(&project) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Takeoff failed: {}", e);
            exit(1);
        }
    };

    println!("Takeoff items: {}", takeoff.items.len());
    for item in &takeoff.items {
        println!(
            "  - [{}] {} ({})",
            item.element_id, item.name, item.category
        );
        for q in &item.quantities {
            println!(
                "      {}: net={} gross={}",
                q.name,
                fmt_quantity(&q.net),
                fmt_quantity(&q.gross())
            );
        }
    }

    // Demonstrate WASM result wrapper at the boundary.
    let _wasm_boundary: WasmResult<()> = WasmResult::ok(());
    let _err_boundary: WasmResult<()> = WasmResult::err("boundary test");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_result_roundtrip() {
        let r: WasmResult<i32> = WasmResult::ok(42);
        assert!(r.ok);
        assert_eq!(r.value, Some(42));

        let e: WasmResult<i32> = WasmResult::err("boom");
        assert!(!e.ok);
        assert_eq!(e.error, Some("boom".to_string()));
    }

    #[test]
    fn import_and_takeoff() {
        let src = include_str!("../../../test-data/ifc/sample.ifc");
        let project = import_ifc(src).expect("import");
        assert_eq!(project.name, "Riverside Office Building");

        let takeoff = run_takeoff(&project).expect("takeoff");
        assert!(!takeoff.items.is_empty());
    }
}
