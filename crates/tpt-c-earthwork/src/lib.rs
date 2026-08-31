// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Earthwork volumes, shrink/swell, mass haul, and volume balancing.
//!
//! Provides the civil/earthwork primitives needed for cut/fill analysis,
//! haul optimization, and volumetric conversions on construction sites.

use serde::{Deserialize, Serialize};
use tpt_c_units::Volume;

/// A swell or shrink ratio expressed as a fraction (0.25 = 25% swell).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwellShrink {
    /// Swell factor when going from bank to loose (e.g. 0.25).
    pub swell: f64,
    /// Shrink factor when going from bank to compacted (e.g. 0.12).
    pub shrink: f64,
}

impl Default for SwellShrink {
    fn default() -> Self {
        Self {
            swell: 0.25,
            shrink: 0.12,
        }
    }
}

/// A single cut/fill station with cumulative volumes.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StationVolume {
    /// Station along the alignment (metres).
    pub station: f64,
    /// Cut volume at this station (m³).
    pub cut: f64,
    /// Fill volume at this station (m³).
    pub fill: f64,
}

impl StationVolume {
    /// Build a station volume record.
    pub fn new(station: f64, cut: f64, fill: f64) -> Self {
        Self { station, cut, fill }
    }

    /// Net volume (positive = excess cut, negative = excess fill).
    pub fn net(&self) -> f64 {
        self.cut - self.fill
    }
}

/// Cumulative earthwork volumes along a corridor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MassHaulDiagram {
    /// Stationed volume records.
    pub stations: Vec<StationVolume>,
    /// Applied swell/shrink factors.
    pub swell_shrink: SwellShrink,
}

impl MassHaulDiagram {
    /// Build an empty mass haul diagram.
    pub fn new(swell_shrink: SwellShrink) -> Self {
        Self {
            stations: Vec::new(),
            swell_shrink,
        }
    }

    /// Add a station record.
    pub fn add_station(&mut self, station: f64, cut: f64, fill: f64) {
        self.stations.push(StationVolume::new(station, cut, fill));
    }

    /// Total cut volume across all stations (bank state).
    pub fn total_cut(&self) -> f64 {
        self.stations.iter().map(|s| s.cut).sum()
    }

    /// Total fill volume across all stations (bank state).
    pub fn total_fill(&self) -> f64 {
        self.stations.iter().map(|s| s.fill).sum()
    }

    /// Net cut after balancing fill (positive = leftover cut to haul away).
    pub fn net_cut(&self) -> f64 {
        self.total_cut() - self.total_fill()
    }

    /// Loose volume equivalent for a bank volume.
    pub fn loose_volume(&self, bank: Volume) -> Volume {
        Volume::from_cubic_meters(bank.cubic_meters() * (1.0 + self.swell_shrink.swell))
    }

    /// Compacted volume equivalent for a bank volume.
    pub fn compacted_volume(&self, bank: Volume) -> Volume {
        Volume::from_cubic_meters(bank.cubic_meters() * (1.0 - self.swell_shrink.shrink))
    }
}

/// Result of a cut/fill balance check.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CutFillBalance {
    /// Total bank cubic metres of cut.
    pub cut_bank: f64,
    /// Total bank cubic metres of fill.
    pub fill_bank: f64,
    /// Excess cut in bank cubic metres (negative = deficit).
    pub excess_cut: f64,
    /// Haul distance factor applied.
    pub haul_factor: f64,
}

impl CutFillBalance {
    /// Compute balance from raw cut/fill station data.
    pub fn compute(cut: f64, fill: f64, haul_factor: f64) -> Self {
        let excess_cut = cut - fill;
        Self {
            cut_bank: cut,
            fill_bank: fill,
            excess_cut,
            haul_factor,
        }
    }

    /// True when there is excess cut to haul.
    pub fn has_excess_cut(&self) -> bool {
        self.excess_cut > 0.0
    }

    /// True when there is a fill deficit.
    pub fn has_deficit(&self) -> bool {
        self.excess_cut < 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_volume_net() {
        let s = StationVolume::new(100.0, 120.0, 80.0);
        assert_eq!(s.net(), 40.0);
    }

    #[test]
    fn mass_haul_totals() {
        let mut mhd = MassHaulDiagram::new(SwellShrink::default());
        mhd.add_station(0.0, 100.0, 60.0);
        mhd.add_station(50.0, 80.0, 90.0);
        assert_eq!(mhd.total_cut(), 180.0);
        assert_eq!(mhd.total_fill(), 150.0);
        assert_eq!(mhd.net_cut(), 30.0);
    }

    #[test]
    fn swell_shrink_volumes() {
        let mhd = MassHaulDiagram::new(SwellShrink {
            swell: 0.25,
            shrink: 0.12,
        });
        let bank = Volume::from_cubic_meters(100.0);
        let loose = mhd.loose_volume(bank);
        let compacted = mhd.compacted_volume(bank);
        assert!((loose.cubic_meters() - 125.0).abs() < 1e-9);
        assert!((compacted.cubic_meters() - 88.0).abs() < 1e-9);
    }

    #[test]
    fn cut_fill_balance() {
        let b = CutFillBalance::compute(200.0, 180.0, 1.0);
        assert!(b.has_excess_cut());
        assert!(!b.has_deficit());
        assert_eq!(b.excess_cut, 20.0);
    }
}
