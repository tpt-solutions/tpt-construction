// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Excel (`.xlsx`) export for estimator workflows.
//!
//! Provides a small, dependency-light writer built over `zip` and `flate2`.
//! The generic [`write_table`] emits a header row plus string/number cells, and
//! [`write_estimate`] produces a bid-ready estimate workbook with a summary
//! block (subtotal, overhead, profit, tax, total).

use std::io::Write;
use std::path::Path;

use tpt_c_cost::{Estimate, Money};
use tpt_c_model::Quantity;

/// Errors raised while writing xlsx.
#[derive(Debug, thiserror::Error)]
pub enum XlsxError {
    /// Underlying IO / ZIP writer failure.
    #[error("xlsx io error: {0}")]
    Io(#[from] std::io::Error),
    /// ZIP archive failure.
    #[error("xlsx zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// Malformed OOXML generated during write.
    #[error("xlsx xml error: {0}")]
    Xml(String),
}

// --- Internal workbook model ---

#[derive(Default)]
struct Workbook {
    sheets: Vec<Sheet>,
}

struct Sheet {
    name: String,
    rows: Vec<Row>,
}

struct Row {
    cells: Vec<Cell>,
}

enum Cell {
    Text(String),
    Number(f64),
}

impl Sheet {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rows: Vec::new(),
        }
    }

    fn add_text_cell(&mut self, text: String) {
        if self.rows.is_empty() {
            self.rows.push(Row::new());
        }
        self.rows.last_mut().unwrap().add_text_cell(text);
    }

    fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }
}

impl Row {
    fn new() -> Self {
        Self { cells: Vec::new() }
    }

    fn add_text_cell(&mut self, text: String) {
        self.cells.push(Cell::Text(text));
    }

    fn add_number_cell(&mut self, value: f64) {
        self.cells.push(Cell::Number(value));
    }
}

impl Workbook {
    fn add_sheet(&mut self, sheet: Sheet) {
        self.sheets.push(sheet);
    }
}

// --- Public API ---

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
    let mut wb = Workbook::default();
    let mut sheet = Sheet::new(sheet_name);
    for h in headers {
        sheet.add_text_cell(h.clone());
    }
    for row in rows {
        let mut r = Row::new();
        for cell in row {
            match cell.parse::<f64>() {
                Ok(n) => r.add_number_cell(n),
                Err(_) => r.add_text_cell(cell.clone()),
            }
        }
        sheet.add_row(r);
    }
    wb.add_sheet(sheet);
    write_workbook(path, wb)
}

/// Write an [`Estimate`] to an `.xlsx` workbook.
///
/// Sheet "Estimate" lists each line item; sheet "Summary" carries the markup
/// breakdown and final total.
pub fn write_estimate<P: AsRef<Path>>(path: P, estimate: &Estimate) -> Result<(), XlsxError> {
    let mut wb = Workbook::default();

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
    let mut est_sheet = Sheet::new("Estimate");
    let headers = [
        "#",
        "Code",
        "Description",
        "Unit",
        "Qty",
        "Unit Rate",
        "Extension",
    ];
    for h in &headers {
        est_sheet.add_text_cell(h.to_string());
    }
    for line in &estimate.line_items {
        let mut r = Row::new();
        r.add_number_cell(line.number as f64);
        r.add_text_cell(line.code.code.clone());
        r.add_text_cell(line.description.clone());
        r.add_text_cell(unit_label(&line.quantity).to_string());
        r.add_number_cell(line.quantity.base_value());
        r.add_number_cell(line.unit_rate.amount());
        r.add_number_cell(line.extension.amount());
        est_sheet.add_row(r);
    }
    wb.add_sheet(est_sheet);

    // --- Summary sheet ---
    let mut sum_sheet = Sheet::new("Summary");
    sum_sheet.add_text_cell("Estimate Summary".to_string());
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
    for (k, v) in &rows {
        let mut r = Row::new();
        r.add_text_cell(k.to_string());
        r.add_text_cell(v.clone());
        sum_sheet.add_row(r);
    }
    wb.add_sheet(sum_sheet);

    write_workbook(path, wb)
}

// --- OOXML generation ---

fn write_workbook<P: AsRef<Path>>(path: P, wb: Workbook) -> Result<(), XlsxError> {
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();

    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(build_content_types(&wb).as_bytes())?;

    zip.start_file("_rels/.rels", options)?;
    zip.write_all(build_root_rels().as_bytes())?;

    zip.start_file("xl/workbook.xml", options)?;
    zip.write_all(build_workbook_xml(&wb).as_bytes())?;

    zip.start_file("xl/_rels/workbook.xml.rels", options)?;
    zip.write_all(build_workbook_rels(&wb).as_bytes())?;

    zip.start_file("xl/styles.xml", options)?;
    zip.write_all(build_styles_xml().as_bytes())?;

    for (i, sheet) in wb.sheets.iter().enumerate() {
        let name = format!("xl/worksheets/sheet{}.xml", i + 1);
        zip.start_file(&name, options)?;
        zip.write_all(build_sheet_xml(sheet).as_bytes())?;
    }

    let _ = zip.finish()?;
    Ok(())
}

fn build_content_types(wb: &Workbook) -> String {
    let mut overrides = String::new();
    overrides.push_str("  <Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\n");
    for i in 0..wb.sheets.len() {
        overrides.push_str(&format!(
            "  <Override PartName=\"/xl/worksheets/sheet{}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>\n",
            i + 1
        ));
    }
    overrides.push_str("  <Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>\n");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
{}
</Types>
"#,
        overrides
    )
}

fn build_root_rels() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>
"#
}

fn build_workbook_xml(wb: &Workbook) -> String {
    let mut sheets = String::new();
    for (i, sheet) in wb.sheets.iter().enumerate() {
        let name = xml_escape(&sheet.name);
        sheets.push_str(&format!(
            "    <sheet name=\"{}\" sheetId=\"{}\" r:id=\"rId{}\"/>\n",
            name,
            i + 1,
            i + 2
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
{}
  </sheets>
</workbook>
"#,
        sheets
    )
}

fn build_workbook_rels(wb: &Workbook) -> String {
    let mut rels = String::new();
    for i in 0..wb.sheets.len() {
        rels.push_str(&format!(
            "  <Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{}.xml\"/>\n",
            i + 1,
            i + 1
        ));
    }
    let styles_id = wb.sheets.len() + 1;
    rels.push_str(&format!(
        "  <Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/>\n",
        styles_id
    ));

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
{}
</Relationships>
"#,
        rels
    )
}

fn build_styles_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border/></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/></cellXfs>
  <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
</styleSheet>
"#
}

fn build_sheet_xml(sheet: &Sheet) -> String {
    let mut rows_xml = String::new();
    for (r_idx, row) in sheet.rows.iter().enumerate() {
        let r = (r_idx + 1).to_string();
        let mut cells_xml = String::new();
        for (c_idx, cell) in row.cells.iter().enumerate() {
            let c = column_label(c_idx as u16);
            let ref_str = format!("{}{}", c, r);
            match cell {
                Cell::Text(text) => {
                    let escaped = xml_escape(text);
                    cells_xml.push_str(&format!(
                        "      <c r=\"{}\" t=\"str\"><v>{}</v></c>\n",
                        ref_str, escaped
                    ));
                }
                Cell::Number(num) => {
                    cells_xml.push_str(&format!("      <c r=\"{}\"><v>{}</v></c>\n", ref_str, num));
                }
            }
        }
        rows_xml.push_str(&format!("    <row r=\"{}\">\n{}    </row>\n", r, cells_xml));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData>
{}
  </sheetData>
</worksheet>
"#,
        rows_xml
    )
}

fn column_label(col: u16) -> String {
    let mut label = String::new();
    let mut c = col;
    while c > 0 {
        c -= 1;
        label.push((b'A' + (c % 26) as u8) as char);
        c /= 26;
    }
    label.chars().rev().collect()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn unit_label(q: &Quantity) -> &'static str {
    match q {
        Quantity::Count(_) => "each",
        Quantity::Length(_) => "m",
        Quantity::Area(_) => "m2",
        Quantity::Volume(_) => "m3",
        Quantity::Mass(_) => "kg",
        Quantity::Duration(_) => "h",
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

    #[test]
    fn xlsx_is_valid_zip() {
        let dir = std::env::temp_dir();
        let path = dir.join("tpt_zip_test.xlsx");
        write_estimate(&path, &sample_estimate()).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let zip = zip::ZipArchive::new(file).unwrap();
        assert!(zip.len() >= 5);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn xlsx_contains_expected_parts() {
        let dir = std::env::temp_dir();
        let path = dir.join("tpt_parts_test.xlsx");
        write_estimate(&path, &sample_estimate()).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let zip = zip::ZipArchive::new(file).unwrap();
        let names: Vec<&str> = zip.file_names().collect();
        assert!(names.contains(&"[Content_Types].xml"));
        assert!(names.contains(&"_rels/.rels"));
        assert!(names.contains(&"xl/workbook.xml"));
        assert!(names.contains(&"xl/styles.xml"));
        assert!(names.iter().any(|n| n.starts_with("xl/worksheets/sheet")));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn xlsx_xml_well_formed() {
        let dir = std::env::temp_dir();
        let path = dir.join("tpt_xml_test.xlsx");
        write_estimate(&path, &sample_estimate()).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();

        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).unwrap();
            let mut buf = Vec::new();
            std::io::copy(&mut entry, &mut buf).unwrap();
            let s = String::from_utf8(buf).unwrap();
            let trimmed = s.trim();
            if trimmed.is_empty() || !trimmed.starts_with("<?xml") {
                continue;
            }
            let mut reader = quick_xml::Reader::from_str(&s);
            let mut unescape_buf = Vec::new();
            loop {
                match reader.read_event(&mut unescape_buf) {
                    Ok(quick_xml::events::Event::Eof) => break,
                    Ok(_) => {}
                    Err(e) => panic!("XML not well-formed in {}: {:?}", entry.name(), e),
                }
            }
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn xlsx_sheet_count_matches() {
        let dir = std::env::temp_dir();
        let path = dir.join("tpt_sheets_test.xlsx");
        write_estimate(&path, &sample_estimate()).unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let zip = zip::ZipArchive::new(file).unwrap();
        let sheet_files: Vec<&str> = zip
            .file_names()
            .filter(|n| n.starts_with("xl/worksheets/sheet") && n.ends_with(".xml"))
            .collect();
        assert_eq!(sheet_files.len(), 2);
        let _ = std::fs::remove_file(&path);
    }
}
