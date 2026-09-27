# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `read_cost_database<R: std::io::Read>()` — parse a cost-database CSV into a
  `tpt_c_cost::CostDatabase`, with strict header validation and per-line errors
- `write_cost_database<W: std::io::Write>()` — export a `CostDatabase` to CSV
- `CostRateRow` record (`code,title,kind,unit,rate,currency`) with `header()`
- Kind/unit mapping to `tpt_c_cost::ResourceKind` / `RateUnit` (case-insensitive)
- Quote-aware line splitting and field parsing: embedded commas, doubled quotes,
  and newlines inside quoted fields, in both directions
- `CsvError` — `Parse`, `Malformed { line, detail }`, `Io`
