// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `tpt` — the TPT Construction command-line interface.
//!
//! Subcommands:
//!
//! ```text
//! tpt estimate <model.json> --cost-db <rates.csv> --output <estimate.xlsx>
//! tpt serve [--addr <addr>] [--token <secret>]   (build with --features serve)
//! ```
//!
//! The model is the neutral `tpt-c-model` JSON (produced by `tpt-c-ifc` and
//! other parsers). Cost rates come from a CSV cost database. The result is a
//! priced estimate written to `.xlsx`. The `serve` subcommand exposes the
//! takeoff, estimate, and CPM scheduling engines over a thin dependency-free
//! HTTP interface (see `src/serve.rs`).

use std::process::ExitCode;

#[cfg(feature = "serve")]
mod serve;

use tpt_c_cost::CostDatabase;
use tpt_c_estimating::EstimateBuilder;
use tpt_c_model::Project;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return usage();
    }
    match args[0].as_str() {
        "estimate" => estimate(&args[1..]),
        #[cfg(feature = "serve")]
        "serve" => serve::run_cli(&args[1..]),
        "help" | "--help" | "-h" => usage(),
        other => {
            eprintln!("unknown subcommand: {other}");
            usage()
        }
    }
}

fn usage() -> ExitCode {
    eprintln!("tpt — TPT Construction CLI");
    eprintln!("  tpt estimate <model.json> --cost-db <rates.csv> --output <estimate.xlsx>");
    eprintln!("  tpt serve [--addr <addr>] [--token <secret>]   (build with --features serve)");
    ExitCode::FAILURE
}

fn estimate(args: &[String]) -> ExitCode {
    let mut model_path: Option<String> = None;
    let mut cost_db: Option<String> = None;
    let mut output: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        match a.as_str() {
            "--cost-db" | "-c" => {
                i += 1;
                cost_db = args.get(i).cloned();
            }
            "--output" | "-o" => {
                i += 1;
                output = args.get(i).cloned();
            }
            other if !other.starts_with('-') && model_path.is_none() => {
                model_path = Some(other.to_string());
            }
            other => {
                eprintln!("unexpected argument: {other}");
                return usage();
            }
        }
        i += 1;
    }

    let model_path = match model_path {
        Some(p) => p,
        None => {
            eprintln!("missing model path");
            return usage();
        }
    };
    let cost_db = match cost_db {
        Some(p) => p,
        None => {
            eprintln!("missing --cost-db");
            return usage();
        }
    };
    let output = match output {
        Some(p) => p,
        None => {
            eprintln!("missing --output");
            return usage();
        }
    };

    let project: Project = match read_json(&model_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let db: CostDatabase = match read_rates(&cost_db) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let builder = EstimateBuilder::new(format!("Estimate — {}", project.name), "USD")
        .for_project(&project)
        .with_database(db);
    let estimate = match builder.from_project(&project) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("estimate failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = tpt_c_xlsx::write_estimate(&output, &estimate) {
        eprintln!("failed to write {output}: {e}");
        return ExitCode::FAILURE;
    }

    match estimate.totals() {
        Ok(t) => println!(
            "Estimate written to {output}: {} line items, subtotal {:.2} {}, total {:.2} {}",
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
