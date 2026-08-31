// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tabular CSV import/export for construction data.
//!
//! Provides round-trippable record types for cost databases and estimates so an
//! estimator can edit rates in a spreadsheet and re-import them. The loader is
//! tolerant of the common columns and ignores unknown trailing fields.

use serde::{Deserialize, Serialize};
use tpt_c_cost::CostDatabase;

/// Errors raised while reading or writing tabular data.
#[derive(Debug, thiserror::Error)]
pub enum CsvError {
    /// Underlying CSV (de)serialization failure.
    #[error("csv io error: {0}")]
    Io(#[from] csv::Error),
    /// A required column was missing or malformed.
    #[error("malformed row {line}: {detail}")]
    Malformed {
        /// Source line number (1-based, including header).
        line: usize,
        /// Human-readable detail.
        detail: String,
    },
}

/// A single row of a cost-database CSV.
///
/// Schema: `code,title,kind,unit,rate,currency`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CostRateRow {
    /// Cost code (e.g. MasterFormat `03 30 00`).
    pub code: String,
    /// Human-readable title.
    pub title: String,
    /// Resource kind: `labor`, `material`, `equipment`, `subcontractor`.
    pub kind: String,
    /// Rate unit: `hour`, `day`, `week`, `each`, `length`, `area`, `volume`, `mass`.
    pub unit: String,
    /// Rate amount per unit.
    pub rate: f64,
    /// ISO 4217 currency code.
    pub currency: String,
}

impl CostRateRow {
    /// Header row for a cost-database CSV.
    pub fn header() -> &'static [&'static str] {
        &["code", "title", "kind", "unit", "rate", "currency"]
    }
}

/// Read a cost database from CSV reader `r`.
pub fn read_cost_database<R: std::io::Read>(r: R) -> Result<CostDatabase, CsvError> {
    let mut reader = csv::Reader::new(r);
    let mut db = CostDatabase::new();
    for (i, result) in reader.deserialize::<CostRateRow>().enumerate() {
        let line = i + 2; // header is line 1
        let row = result.map_err(|e| CsvError::Malformed {
            line,
            detail: e.to_string(),
        })?;
        let kind = parse_kind(&row.kind).ok_or_else(|| CsvError::Malformed {
            line,
            detail: format!("unknown resource kind '{}'", row.kind),
        })?;
        let unit = parse_unit(&row.unit).ok_or_else(|| CsvError::Malformed {
            line,
            detail: format!("unknown rate unit '{}'", row.unit),
        })?;
        db.add_rate(
            row.code.clone(),
            row.code.clone(),
            kind,
            unit,
            tpt_c_cost::Money::new(row.rate, row.currency.clone()),
        );
        let _ = row.title;
    }
    Ok(db)
}

/// Write a cost database to CSV writer `w`.
pub fn write_cost_database<W: std::io::Write>(w: W, db: &CostDatabase) -> Result<(), CsvError> {
    let mut writer = csv::Writer::from_writer(w);
    writer.write_record(CostRateRow::header())?;
    for rate in db.rates() {
        writer.serialize(CostRateRow {
            code: rate.id.clone(),
            title: rate.description.clone(),
            kind: kind_name(rate.kind).to_string(),
            unit: unit_name(rate.unit).to_string(),
            rate: rate.rate.amount(),
            currency: rate.rate.currency().to_string(),
        })?;
    }
    writer.flush()?;
    Ok(())
}

fn parse_kind(s: &str) -> Option<tpt_c_cost::ResourceKind> {
    match s.trim().to_ascii_lowercase().as_str() {
        "labor" => Some(tpt_c_cost::ResourceKind::Labor),
        "material" => Some(tpt_c_cost::ResourceKind::Material),
        "equipment" => Some(tpt_c_cost::ResourceKind::Equipment),
        "subcontractor" => Some(tpt_c_cost::ResourceKind::Subcontractor),
        _ => None,
    }
}

fn kind_name(k: tpt_c_cost::ResourceKind) -> &'static str {
    match k {
        tpt_c_cost::ResourceKind::Labor => "labor",
        tpt_c_cost::ResourceKind::Material => "material",
        tpt_c_cost::ResourceKind::Equipment => "equipment",
        tpt_c_cost::ResourceKind::Subcontractor => "subcontractor",
    }
}

fn parse_unit(s: &str) -> Option<tpt_c_cost::RateUnit> {
    match s.trim().to_ascii_lowercase().as_str() {
        "hour" => Some(tpt_c_cost::RateUnit::Hour),
        "day" => Some(tpt_c_cost::RateUnit::Day),
        "week" => Some(tpt_c_cost::RateUnit::Week),
        "each" => Some(tpt_c_cost::RateUnit::Each),
        "length" => Some(tpt_c_cost::RateUnit::Length),
        "area" => Some(tpt_c_cost::RateUnit::Area),
        "volume" => Some(tpt_c_cost::RateUnit::Volume),
        "mass" => Some(tpt_c_cost::RateUnit::Mass),
        _ => None,
    }
}

fn unit_name(u: tpt_c_cost::RateUnit) -> &'static str {
    match u {
        tpt_c_cost::RateUnit::Hour => "hour",
        tpt_c_cost::RateUnit::Day => "day",
        tpt_c_cost::RateUnit::Week => "week",
        tpt_c_cost::RateUnit::Each => "each",
        tpt_c_cost::RateUnit::Length => "length",
        tpt_c_cost::RateUnit::Area => "area",
        tpt_c_cost::RateUnit::Volume => "volume",
        tpt_c_cost::RateUnit::Mass => "mass",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_cost::Money;

    #[test]
    fn cost_db_roundtrip() {
        let mut db = CostDatabase::new();
        db.add_rate(
            "03 30 00",
            "03 30 00",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Volume,
            Money::new(120.0, "USD"),
        );
        let mut buf: Vec<u8> = Vec::new();
        write_cost_database(&mut buf, &db).unwrap();
        let back = read_cost_database(buf.as_slice()).unwrap();
        assert_eq!(back.get("03 30 00").unwrap().rate.amount(), 120.0);
    }

    #[test]
    fn rejects_bad_kind() {
        let csv = "code,title,kind,unit,rate,currency\nA,bad,robot,each,1,USD\n";
        assert!(read_cost_database(csv.as_bytes()).is_err());
    }
}
