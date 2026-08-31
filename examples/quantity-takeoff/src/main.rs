// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Quantity takeoff example.
//!
//! Reads a neutral model (tpt-c-model JSON), runs the takeoff engine, and prints
//! a per-element quantity summary plus gross totals by kind.

use std::process::ExitCode;

use tpt_c_model::Project;
use tpt_c_quantities::{QuantityKind, TakeoffEngine};

fn main() -> ExitCode {
    let path = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: quantity-takeoff <model.json>");
            return ExitCode::FAILURE;
        }
    };
    let json = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to read {path}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let project: Project = match serde_json::from_str(&json) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("failed to parse model: {e}");
            return ExitCode::FAILURE;
        }
    };

    let result = TakeoffEngine::new().run(&project);
    println!(
        "Takeoff for '{}': {} elements",
        project.name,
        result.items.len()
    );
    for item in &result.items {
        println!("\n[{}] {} ({})", item.element_id, item.name, item.category);
        for q in &item.quantities {
            println!(
                "  - {}: net {:.4}  gross {:.4}  waste {:.1}%{}",
                q.name,
                q.net.base_value(),
                q.gross().base_value(),
                q.waste.ratio() * 100.0,
                if q.manual { "  [override]" } else { "" }
            );
        }
    }

    print_total("Volume", QuantityKind::Volume, &result);
    print_total("Area", QuantityKind::Area, &result);
    print_total("Length", QuantityKind::Length, &result);
    print_total("Mass", QuantityKind::Mass, &result);
    print_total("Count", QuantityKind::Count, &result);
    ExitCode::SUCCESS
}

fn print_total(label: &str, kind: QuantityKind, result: &tpt_c_quantities::TakeoffResult) {
    let net = result.total_net(kind);
    let gross = result.total_gross(kind);
    println!("\nTotal {label}: net {:.4}  gross {:.4}", net, gross);
}
