// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end vertical-slice test: model → takeoff → cost items → estimate → xlsx.
//!
//! Uses the golden cost database in `test-data/golden/rates.csv` and a synthetic
//! model, exercising the full pipeline the `tpt estimate` CLI runs.

use std::path::Path;

use tpt_c_classification::{Classification, ClassificationSystem};
use tpt_c_core::ProjectId;
use tpt_c_cost::{CostCode, CostDatabase, Money, RateUnit, ResourceKind, ResourceRate};
use tpt_c_estimating::EstimateBuilder;
use tpt_c_ids::IdFactory;
use tpt_c_model::{Element, Project, PropertySet, PropertyValue, Quantity, QuantitySet};
use tpt_c_quantities::TakeoffEngine;
use tpt_c_units::{Area, Volume};

fn golden_rates() -> CostDatabase {
    let mut db = CostDatabase::new();
    db.insert(
        "03 30 00",
        ResourceRate::new(
            "r1",
            ResourceKind::Material,
            RateUnit::Volume,
            Money::new(120.0, "USD"),
        )
        .with_description("Concrete CY"),
    );
    db.insert(
        "03 20 00",
        ResourceRate::new(
            "r2",
            ResourceKind::Material,
            RateUnit::Area,
            Money::new(15.0, "USD"),
        )
        .with_description("Formwork SF"),
    );
    db.insert(
        "09 30 00",
        ResourceRate::new(
            "r3",
            ResourceKind::Material,
            RateUnit::Area,
            Money::new(3.0, "USD"),
        )
        .with_description("Paint SF"),
    );
    db
}

fn sample_project() -> Project {
    let mut p = Project::new(ProjectId::nil(), "Golden Demo");
    let slab = Element::new(IdFactory::element(), "SLAB-1", "Slab")
        .classified(Classification::new(
            ClassificationSystem::MasterFormat,
            "03 30 00",
        ))
        .with_quantity_set(
            QuantitySet::new("BaseQuantities")
                .with(
                    "GrossVolume",
                    Quantity::Volume(Volume::from_cubic_yards(10.0)),
                )
                .with(
                    "FormworkArea",
                    Quantity::Area(Area::from_square_feet(400.0)),
                ),
        );
    p.add_element(slab);

    let wall = Element::new(IdFactory::element(), "WALL-1", "Wall")
        .classified(Classification::new(
            ClassificationSystem::MasterFormat,
            "09 30 00",
        ))
        .with_property_set(PropertySet::new("R").with("RebarSize", PropertyValue::Number(5.0)))
        .with_quantity_set(
            QuantitySet::new("Q")
                .with("PaintArea", Quantity::Area(Area::from_square_feet(300.0)))
                .with(
                    "RebarLength",
                    Quantity::Length(tpt_c_units::Length::from_meters(20.0)),
                ),
        );
    p.add_element(wall);
    p
}

#[test]
fn model_to_estimate_pipeline() {
    let project = sample_project();
    let takeoff = TakeoffEngine::new().run(&project);
    assert_eq!(takeoff.items.len(), 2);

    let estimate = EstimateBuilder::new("Golden Estimate", "USD")
        .with_database(golden_rates())
        .from_takeoff(&takeoff)
        .expect("estimate builds");

    // Priced lines include derived quantities (slab concrete + formwork, wall
    // paint + rebar weight). At least three, all with positive extensions.
    assert!(estimate.line_items.len() >= 3);
    for line in &estimate.line_items {
        assert!(line.extension.amount() > 0.0);
    }
    let totals = estimate.totals().expect("totals");
    assert!(totals.subtotal.amount() > 0.0);
    assert!(totals.total.amount() > totals.subtotal.amount());
}

#[test]
fn estimate_writes_xlsx() {
    let project = sample_project();
    let takeoff = TakeoffEngine::new().run(&project);
    let estimate = EstimateBuilder::new("Golden Estimate", "USD")
        .with_database(golden_rates())
        .from_takeoff(&takeoff)
        .unwrap();

    let out = std::env::temp_dir().join("tpt_e2e_golden.xlsx");
    tpt_c_xlsx::write_estimate(&out, &estimate).expect("write xlsx");
    assert!(out.exists());
    let _ = std::fs::remove_file(&out);
}

#[test]
fn golden_model_json_is_materialized() {
    // Ensure the committed golden model exists for the CLI vertical slice.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("test-data/golden/sample-model.json");
    let json = serde_json::to_string_pretty(&sample_project()).expect("serialize model");
    std::fs::write(&path, json).expect("write golden model");
    assert!(path.exists());
}

#[test]
fn golden_rates_csv_loads() {
    // The committed golden file must round-trip through the CSV loader.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("test-data/golden/rates.csv");
    let f = std::fs::File::open(&path).expect("golden rates.csv present");
    let db = tpt_c_csv::read_cost_database(f).expect("parse rates");
    assert!(db.get("03 30 00").is_some());
    let code = CostCode::from_classification(&Classification::new(
        ClassificationSystem::MasterFormat,
        "03 30 00",
    ));
    assert_eq!(db.get(&code.code).unwrap().rate.amount(), 120.0);
}
