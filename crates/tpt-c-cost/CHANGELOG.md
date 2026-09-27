# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Money` — currency-carrying amounts with `new`, `zero`, `amount`, `currency`,
  `rounded`, `checked_add`/`checked_sub`, `scaled`, and `Add`/`Sub`/`Mul<f64>`
  operators
- `ResourceKind` (labor/material/equipment/subcontractor) and `RateUnit`
  (hour/day/week/each/length/area/volume/mass)
- `ResourceRate` with builder-style `with_description`
- `CostCode` (`new`, `with_title`, `from_classification`) and `CostDatabase`
  (`insert`, `get`, `rates`, `len`, `is_empty`, `add_rate`)
- `CostItem`, `CostAssembly` (rolled-up `total()`), and `LineItem`
  (`new`, `from_item`) with computed extensions
- `Markup` waterfall (overhead → profit → escalation → tax) with builder setters
  and `totals()`; `MarkupTotals` breakdown
- `Estimate` — `new`, `add_line`, `with_markup`, `with_notes`, `subtotal`,
  `totals`, `total`; implements `Identified`
- `Budget` — allowance + contingency with `variance()` comparison
- `round_money` helper and `CostError` (`CurrencyMismatch`, `MissingRate`,
  `MissingQuantity`)
