// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Excel (`.xlsx`) export for estimator workflows.
//!
//! Provides a small, dependency-light writer built on `rust_xlsxwriter`. The
//! generic [`write_table`] emits a header row plus string/number cells, and
//! [`write_estimate`] produces a bid-ready estimate workbook with a summary
//! block (subtotal, overhead, profit, tax, total).

use std::path::Path;

use rust_xlsxwriter::{Format, Workbook, Worksheet};
use tpt_c_cost::{Estimate, Money};

/// Errors raised while writing xlsx.
#[derive(Debug, thiserror::Error)]
pub enum XlsxError {
    /// Underlying workbook IO / writer failure.
    #[error("xlsx error: {0}")]
    Writer(#[from] rust_xlsxwriter::XlsxError),
}

/// Write a sheet named `sheet_name` containing `headers` followed by `rows`.
///
/// Each cell is written as a string; numeric strings are emitted as numbers so
/// Excel treats them as values.
pub fn write_table<P: AsRef<Path>>(
    path: P,
    sheet_name: &str,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    if !sheet_name.is_empty() {
        let _ = worksheet.set_name(sheet_name);
    }

    let header_fmt = Format::new().set_bold();
    for (c, h) in headers.iter().enumerate() {
        worksheet.write_with_format(0, c as u16, h.as_str(), &header_fmt)?;
    }
    for (r, row) in rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            write_cell(worksheet, (r + 1) as u32, c as u16, cell)?;
        }
    }
    workbook.save(path)?;
    Ok(())
}

/// Write an [`Estimate`] to an `.xlsx` workbook.
///
/// Sheet "Estimate" lists each line item; sheet "Summary" carries the markup
/// breakdown and final total.
pub fn write_estimate<P: AsRef<Path>>(path: P, estimate: &Estimate) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let summary = estimate.totals().unwrap_or_else(|_| {
        let cur = estimate.currency.clone();
        tpt_c_cost::MarkupTotals {
            subtotal: Money::zero(&cur),
            overhead: Money::zero(&cur),
            profit: Money::zero(&cur),
            escalation: Money::zero(&cur),
            tax: Money::zero(&cur),
            total: Money::zero(&cur),
        }
    });

    // --- Estimate sheet ---
    let ws = workbook.add_worksheet();
    let _ = ws.set_name("Estimate");
    let header_fmt = Format::new().set_bold();
    let headers = [
        "#",
        "Code",
        "Description",
        "Unit",
        "Qty",
        "Unit Rate",
        "Extension",
    ];
    for (c, h) in headers.iter().enumerate() {
        ws.write_with_format(0, c as u16, *h, &header_fmt)?;
    }
    for (i, line) in estimate.line_items.iter().enumerate() {
        let r = (i + 1) as u32;
        ws.write(r, 0, line.number as f64)?;
        ws.write(r, 1, line.code.code.as_str())?;
        ws.write(r, 2, line.description.as_str())?;
        ws.write(r, 3, unit_label(&line.quantity))?;
        ws.write(r, 4, line.quantity.base_value())?;
        ws.write(r, 5, line.unit_rate.amount())?;
        ws.write(r, 6, line.extension.amount())?;
    }

    // --- Summary sheet ---
    let ss = workbook.add_worksheet();
    let _ = ss.set_name("Summary");
    ss.write_with_format(0, 0, "Estimate Summary", &header_fmt)?;
    let rows = [
        ("Title", estimate.title.clone()),
        ("Currency", estimate.currency.clone()),
        ("Subtotal", money_str(summary.subtotal)),
        ("Overhead", money_str(summary.overhead)),
        ("Profit", money_str(summary.profit)),
        ("Escalation", money_str(summary.escalation)),
        ("Tax", money_str(summary.tax)),
        ("Total", money_str(summary.total)),
    ];
    for (i, (k, v)) in rows.iter().enumerate() {
        let r = (i + 1) as u32;
        ss.write(r, 0, *k)?;
        ss.write(r, 1, v.as_str())?;
    }

    workbook.save(path)?;
    Ok(())
}

fn write_cell(ws: &mut Worksheet, r: u32, c: u16, value: &str) -> Result<(), XlsxError> {
    match value.parse::<f64>() {
        Ok(n) => {
            ws.write(r, c, n)?;
        }
        Err(_) => {
            ws.write(r, c, value)?;
        }
    }
    Ok(())
}

fn unit_label(q: &tpt_c_model::Quantity) -> &'static str {
    match q {
        tpt_c_model::Quantity::Count(_) => "each",
        tpt_c_model::Quantity::Length(_) => "m",
        tpt_c_model::Quantity::Area(_) => "m2",
        tpt_c_model::Quantity::Volume(_) => "m3",
        tpt_c_model::Quantity::Mass(_) => "kg",
        tpt_c_model::Quantity::Duration(_) => "h",
    }
}

fn money_str(m: Money) -> String {
    format!("{:.2} {}", m.amount(), m.currency())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_cost::{CostCode, LineItem, Markup, Money};
    use tpt_c_model::Quantity;
    use tpt_c_units::Volume;

    fn sample_estimate() -> Estimate {
        let mut e = Estimate::new("Demo", "USD").with_markup(Markup::none());
        e.add_line(LineItem::new(
            1,
            CostCode::new("03 30 00"),
            Quantity::Volume(Volume::from_cubic_yards(10.0)),
            Money::new(120.0, "USD"),
        ));
        e
    }

    #[test]
    fn write_estimate_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("tpt_estimate_test.xlsx");
        write_estimate(&path, &sample_estimate()).unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn write_table_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("tpt_table_test.xlsx");
        let headers = vec!["Code".to_string(), "Rate".to_string()];
        let rows = vec![vec!["03 30 00".to_string(), "120.5".to_string()]];
        write_table(&path, "Rates", &headers, &rows).unwrap();
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }
}
