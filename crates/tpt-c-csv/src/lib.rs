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
    #[error("csv parse error: {0}")]
    Parse(String),
    /// A required column was missing or malformed.
    #[error("malformed row {line}: {detail}")]
    Malformed {
        /// Source line number (1-based, including header).
        line: usize,
        /// Human-readable detail.
        detail: String,
    },
    /// Raw IO failure (e.g. flush).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
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
    #[allow(dead_code)]
    pub fn header() -> &'static [&'static str] {
        &["code", "title", "kind", "unit", "rate", "currency"]
    }
}

/// Read a cost database from CSV reader `r`.
pub fn read_cost_database<R: std::io::Read>(mut r: R) -> Result<CostDatabase, CsvError> {
    let mut buf = String::new();
    r.read_to_string(&mut buf).map_err(CsvError::Io)?;
    let lines = csv_lines(&buf);

    let header = lines
        .first()
        .ok_or_else(|| CsvError::Parse("empty input".into()))?;
    let expected = CostRateRow::header().join(",");
    if *header != expected {
        return Err(CsvError::Parse(format!(
            "unexpected header: expected '{}', got '{}'",
            expected, header
        )));
    }

    let mut db = CostDatabase::new();
    for (i, line) in lines.iter().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let line_num = i + 2; // header is line 1
        let fields = parse_csv_line(line)?;
        if fields.len() != 6 {
            return Err(CsvError::Malformed {
                line: line_num,
                detail: format!("expected 6 fields, got {}", fields.len()),
            });
        }

        let row = CostRateRow {
            code: fields[0].clone(),
            title: fields[1].clone(),
            kind: fields[2].clone(),
            unit: fields[3].clone(),
            rate: fields[4].parse().map_err(|e| CsvError::Malformed {
                line: line_num,
                detail: format!("invalid rate '{}': {}", fields[4], e),
            })?,
            currency: fields[5].clone(),
        };

        let kind = parse_kind(&row.kind).ok_or_else(|| CsvError::Malformed {
            line: line_num,
            detail: format!("unknown resource kind '{}'", row.kind),
        })?;
        let unit = parse_unit(&row.unit).ok_or_else(|| CsvError::Malformed {
            line: line_num,
            detail: format!("unknown rate unit '{}'", row.unit),
        })?;
        let mut rate = tpt_c_cost::ResourceRate::new(
            row.code.clone(),
            kind,
            unit,
            tpt_c_cost::Money::new(row.rate, row.currency.clone()),
        );
        rate = rate.with_description(row.title.clone());
        db.insert(row.code.clone(), rate);
    }
    Ok(db)
}

/// Write a cost database to CSV writer `w`.
pub fn write_cost_database<W: std::io::Write>(mut w: W, db: &CostDatabase) -> Result<(), CsvError> {
    writeln!(w, "{}", CostRateRow::header().join(","))?;
    for rate in db.rates() {
        let row = CostRateRow {
            code: rate.id.clone(),
            title: rate.description.clone(),
            kind: kind_name(rate.kind).to_string(),
            unit: unit_name(rate.unit).to_string(),
            rate: rate.rate.amount(),
            currency: rate.rate.currency().to_string(),
        };
        writeln!(
            w,
            "{},{},{},{},{},{}",
            escape_csv_field(&row.code),
            escape_csv_field(&row.title),
            escape_csv_field(&row.kind),
            escape_csv_field(&row.unit),
            escape_csv_field(&row.rate.to_string()),
            escape_csv_field(&row.currency),
        )?;
    }
    Ok(())
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace("\"", "\"\""))
    } else {
        field.to_string()
    }
}

fn csv_lines(input: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut line_start = 0;
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut in_quotes = false;

    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                if in_quotes && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                    i += 1; // skip escaped quote
                } else {
                    in_quotes = !in_quotes;
                }
            }
            b'\n' | b'\r' if !in_quotes => {
                let end = i;
                lines.push(&input[line_start..end]);
                if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                    line_start = i + 2;
                    i += 1;
                } else {
                    line_start = i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if line_start <= bytes.len() {
        lines.push(&input[line_start..]);
    }

    lines
}

fn parse_csv_line(line: &str) -> Result<Vec<String>, CsvError> {
    let bytes = line.as_bytes();
    let mut fields = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        let mut field = String::new();

        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                        field.push('"');
                        i += 2;
                    } else {
                        i += 1;
                        break;
                    }
                } else {
                    field.push(bytes[i] as char);
                    i += 1;
                }
            }
        } else {
            while i < bytes.len() && bytes[i] != b',' {
                field.push(bytes[i] as char);
                i += 1;
            }
        }

        fields.push(field);

        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
        }
    }

    Ok(fields)
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
    use tpt_c_cost::{Money, ResourceRate};

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

    #[test]
    fn roundtrip_with_commas_in_title() {
        let mut db = CostDatabase::new();
        let mut rate = ResourceRate::new(
            "09 21 00",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Area,
            Money::new(45.0, "USD"),
        );
        rate = rate.with_description("Acoustic Tile, 2x4");
        db.insert("09 21 00", rate);

        let mut buf: Vec<u8> = Vec::new();
        write_cost_database(&mut buf, &db).unwrap();
        let back = read_cost_database(buf.as_slice()).unwrap();
        assert_eq!(back.get("09 21 00").unwrap().rate.amount(), 45.0);
        assert_eq!(
            back.get("09 21 00").unwrap().description,
            "Acoustic Tile, 2x4"
        );
    }

    #[test]
    fn roundtrip_with_quotes_in_title() {
        let mut db = CostDatabase::new();
        let mut rate = ResourceRate::new(
            "09 21 00",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Area,
            Money::new(45.0, "USD"),
        );
        rate = rate.with_description("Tile says \"hello\"");
        db.insert("09 21 00", rate);

        let mut buf: Vec<u8> = Vec::new();
        write_cost_database(&mut buf, &db).unwrap();
        let back = read_cost_database(buf.as_slice()).unwrap();
        assert_eq!(
            back.get("09 21 00").unwrap().description,
            "Tile says \"hello\""
        );
    }

    #[test]
    fn roundtrip_with_newlines_in_title() {
        let mut db = CostDatabase::new();
        let mut rate = ResourceRate::new(
            "09 21 00",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Area,
            Money::new(45.0, "USD"),
        );
        rate = rate.with_description("Line1\nLine2");
        db.insert("09 21 00", rate);

        let mut buf: Vec<u8> = Vec::new();
        write_cost_database(&mut buf, &db).unwrap();
        let back = read_cost_database(buf.as_slice()).unwrap();
        assert_eq!(back.get("09 21 00").unwrap().description, "Line1\nLine2");
    }

    #[test]
    fn writer_quotes_fields_with_commas() {
        let mut db = CostDatabase::new();
        let mut rate = ResourceRate::new(
            "09 21 00",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Area,
            Money::new(45.0, "USD"),
        );
        rate = rate.with_description("Acoustic Tile, 2x4");
        db.insert("09 21 00", rate);

        let mut buf: Vec<u8> = Vec::new();
        write_cost_database(&mut buf, &db).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"Acoustic Tile, 2x4\""));
    }

    #[test]
    fn writer_escapes_quotes() {
        let mut db = CostDatabase::new();
        let mut rate = ResourceRate::new(
            "09 21 00",
            tpt_c_cost::ResourceKind::Material,
            tpt_c_cost::RateUnit::Area,
            Money::new(45.0, "USD"),
        );
        rate = rate.with_description("Tile says \"hello\"");
        db.insert("09 21 00", rate);

        let mut buf: Vec<u8> = Vec::new();
        write_cost_database(&mut buf, &db).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("\"Tile says \"\"hello\"\"\""));
    }
}
