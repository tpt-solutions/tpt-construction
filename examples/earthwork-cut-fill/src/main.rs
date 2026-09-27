// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Phase 5 integration example: stake out a corridor alignment, compare the
//! proposed profile against synthetic existing ground, build a mass haul
//! diagram, and report the cut/fill balance.
//!
//! Run with `cargo run -p earthwork-cut-fill`.

use tpt_c_alignment::{HorizontalElement, Point2D, Station, Superelevation, VerticalElement};
use tpt_c_earthwork::{CutFillBalance, MassHaulDiagram, SwellShrink};
use tpt_c_units::Volume;

/// Corridor width contributing to earthwork volumes (metres).
const CORRIDOR_WIDTH_M: f64 = 12.0;
/// Distance between sampled stations (metres).
const STATION_INTERVAL_M: f64 = 25.0;

fn main() {
    // --- Horizontal alignment: tangent, circular curve, tangent -------------
    let horiz = [
        HorizontalElement::Tangent {
            start: Point2D::new(0.0, 0.0),
            end: Point2D::new(300.0, 0.0),
        },
        HorizontalElement::Circular {
            start: Point2D::new(300.0, 0.0),
            center: Point2D::new(300.0, 200.0),
            radius: 200.0,
            sweep: std::f64::consts::FRAC_PI_2,
        },
        HorizontalElement::Tangent {
            start: Point2D::new(500.0, 200.0),
            end: Point2D::new(800.0, 200.0),
        },
    ];
    let plan_length: f64 = horiz.iter().map(|e| e.length().meters()).sum();

    // --- Vertical alignment: climb, crest curve, descend --------------------
    let profile = [
        VerticalElement::Grade {
            start_elevation: 100.0,
            grade: 0.04,
            length: 300.0,
        },
        VerticalElement::VerticalCurve {
            start_elevation: 112.0,
            grade_in: 0.04,
            grade_out: -0.03,
            length: 140.0,
        },
        VerticalElement::Grade {
            start_elevation: 114.2,
            grade: -0.03,
            length: plan_length - 440.0,
        },
    ];

    // --- Sample stations and compare proposed vs existing ground ------------
    // Synthetic rolling existing ground for the demonstration. The sine wave
    // crosses the proposed profile twice, so the corridor contains both cut
    // and fill.
    let existing_ground = |d: f64| 105.0 + 8.0 * (d / 230.0).sin();

    let mut mass_haul = MassHaulDiagram::new(SwellShrink {
        swell: 0.25,  // bank -> loose grows 25%
        shrink: 0.10, // bank -> compacted shrinks 10%
    });

    let mut d = 0.0;
    while d <= plan_length {
        let proposed = profile_elevation(&profile, d);
        let existing = existing_ground(d);
        // Cut depth (existing above proposed) or fill depth (proposed above
        // existing), converted to bank volume over the station interval.
        let depth = existing - proposed;
        let volume = depth * CORRIDOR_WIDTH_M * STATION_INTERVAL_M;
        let (cut, fill) = if volume > 0.0 {
            (volume, 0.0)
        } else {
            (0.0, -volume)
        };
        mass_haul.add_station(d, cut, fill);
        d += STATION_INTERVAL_M;
    }

    // --- Report --------------------------------------------------------------
    println!("Corridor plan length: {:.1} m", plan_length);
    for (i, e) in horiz.iter().enumerate() {
        println!("  horizontal element {i}: {:.1} m", e.length().meters());
    }
    let station = Station::from_metres(450);
    println!(
        "Station {}: proposed elevation {:.2} m (existing {:.2} m)",
        station.format(),
        profile_elevation(&profile, 450.0),
        existing_ground(450.0)
    );

    let se = Superelevation::new(0.06, 100.0, -0.02);
    println!(
        "Superelevation at 60 m into runoff: {:.1}%",
        se.rate_at(60.0) * 100.0
    );

    println!("\nMass haul ({} stations):", mass_haul.stations.len());
    let cut = mass_haul.total_cut();
    let fill = mass_haul.total_fill();
    println!("  total cut:  {:>10.1} bank m3", cut);
    println!("  total fill: {:>10.1} bank m3", fill);
    println!("  net (cut-fill): {:>7.1} bank m3", mass_haul.net_cut());

    let cut_bank = Volume::from_cubic_meters(cut);
    println!(
        "  cut as loose volume: {:.1} m3",
        mass_haul.loose_volume(cut_bank).cubic_meters()
    );
    println!("  cut in cubic yards:  {:.1} CY", cut_bank.cubic_yards());

    let balance = CutFillBalance::compute(cut, fill, 1.15);
    println!("\nBalance (haul factor {:.2}):", balance.haul_factor);
    println!("  excess cut: {:.1} bank m3", balance.excess_cut);
    if balance.has_excess_cut() {
        println!("  => surplus cut must be hauled to spoil or stockpiled");
    } else if balance.has_deficit() {
        println!("  => borrow material required to make up the fill deficit");
    } else {
        println!("  => earthwork balances on site");
    }
}

/// Proposed profile elevation at distance `d` along the alignment.
fn profile_elevation(profile: &[VerticalElement], mut d: f64) -> f64 {
    for element in profile {
        let len = element.length().meters();
        if d <= len {
            return element.elevation_at(d);
        }
        d -= len;
    }
    // Past the end: continue on the last element's tangent.
    profile
        .last()
        .map(|e| e.elevation_at(e.length().meters()))
        .unwrap_or(0.0)
}
