# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `ChangeOrder` with `new`, `with_description`, `with_cost_impact`, `with_time_impact`, `add_line`, and `transition`, plus the `ChangeLine` scope line type.
- `ChangeOrderStatus` lifecycle state machine (`Draft` through `Implemented`/`Withdrawn`) with `is_terminal` and `is_approved`.
- `ChangeRegister` per-project ledger: `add`, `get`, `get_mut`, `transition`, `orders`, `len`, `is_empty`, `approved_cost_impact`, `approved_time_impact_days`, and `pending`.
- Revision diffing: `RevisionDiff::compute` and `net_cost_delta`, `QuantityDiff::compute` and `total_absolute_delta`, `QuantityDelta::delta`, `QuantityEntry`, `CostDelta::delta`, and the `DeltaKind` classifier.
- `ChangeError` (`InvalidTransition`, `DuplicateNumber`, `DuplicateId`, `UnknownChangeOrder`, `CurrencyMismatch`) with conversion from `tpt_c_cost::CostError`.
