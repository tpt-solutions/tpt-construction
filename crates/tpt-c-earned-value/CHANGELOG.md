# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Amount` monetary type (value + ISO 4217 currency) with `new`, `value`, `currency`, `checked_add`, `checked_sub`, and currency-checked `Add`/`Sub`.
- `EarnedValueStatus::new` (project, BAC, BCWS, BCWP, ACWP) with single-currency validation.
- Variance metrics: `schedule_variance` and `cost_variance`.
- Performance indices: `spi`, `cpi`, `percent_complete`.
- Forecasts: `eac`, `etc`, and `vac` parameterised by `EacMethod` (`Typical`, `Atypical`, `Combined`).
- To-complete indices: `tcpib` and `tcpie`.
- `EarnedValueError::CurrencyMismatch`.
