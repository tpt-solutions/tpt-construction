# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `write_table<P: AsRef<Path>>(path, sheet_name, headers, rows)` — generic
  single-sheet xlsx writer; numeric strings become number cells
- `write_estimate<P: AsRef<Path>>(path, &Estimate)` — two-sheet estimate
  workbook: "Estimate" line items and "Summary" markup breakdown (subtotal,
  overhead, profit, escalation, tax, total)
- Minimal hand-rolled OOXML package generation over `zip` + `flate2`
  (content types, relationships, workbook, styles, worksheets)
- XML escaping for sheet names and text cells; A1-style cell references via
  column-label computation
- `XlsxError` — `Io`, `Zip`, `Xml`
