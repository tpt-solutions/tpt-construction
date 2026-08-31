// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Estimate export example.
//!
//! Reads a neutral model (tpt-c-model JSON) and a cost-database CSV, builds a
//! priced estimate, and writes it to an `.xlsx` workbook.

use std::process::ExitCode;

use tpt_c_cost::CostDatabase;
use tpt_c_estimating::EstimateBuilder;
use tpt_c_model::Project;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let model_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: estimate-export <model.json> <rates.csv> <out.xlsx>");
            return ExitCode::FAILURE;
        }
    };
    let rates_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("missing cost-database csv argument");
            return ExitCode::FAILURE;
        }
    };
    let out_path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("missing output xlsx path");
            return ExitCode::FAILURE;
        }
    };

    let project: Project = match read_json(&model_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let db = match read_rates(&rates_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let builder = EstimateBuilder::new(
        format!("Estimate — {}", project.name),
        project_currency(&project),
    )
    .for_project(&project)
    .with_database(db);
    let estimate = match builder.from_project(&project) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("estimate failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = tpt_c_xlsx::write_estimate(&out_path, &estimate) {
        eprintln!("failed to write {out_path}: {e}");
        return ExitCode::FAILURE;
    }

    match estimate.totals() {
        Ok(t) => println!(
            "Wrote {out_path}: {} line items, subtotal {:.2} {}, total {:.2} {}",
            estimate.line_items.len(),
            t.subtotal.amount(),
            estimate.currency,
            t.total.amount(),
            estimate.currency
        ),
        Err(e) => eprintln!("estimate totals unavailable: {e}"),
    }
    ExitCode::SUCCESS
}

fn read_json(path: &str) -> Result<Project, String> {
    let s = std::fs::read_to_string(path).map_err(|e| format!("read {path}: {e}"))?;
    serde_json::from_str(&s).map_err(|e| format!("parse {path}: {e}"))
}

fn read_rates(path: &str) -> Result<CostDatabase, String> {
    let f = std::fs::File::open(path).map_err(|e| format!("open {path}: {e}"))?;
    tpt_c_csv::read_cost_database(f).map_err(|e| format!("read rates: {e}"))
}

fn project_currency(project: &Project) -> String {
    // The neutral model carries no currency; default to USD for the export.
    let _ = project;
    "USD".to_string()
}
