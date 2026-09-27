# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `EstimateBuilder` with `new`, `with_database`, `with_markup`, `price_gross`, `for_project`, `from_project`, and `from_takeoff` to price a takeoff into an `Estimate`.
- `PricingError` (`MissingDatabase`, `MissingRate`, `CurrencyMismatch`) for pricing failures.
- `BidPreparation` bid wrapper with `new` and `total`.
- `EstimateRevision` with `new`, `total`, and `compare`, returning an `EstimateComparison` (total delta, line-count delta, per-code deltas).
- `CostPlan` / `CostPlanRow` by-cost-code budget rollup with `from_estimate` and `total`.
- Convenience helpers `code_for` (classification or category to `CostCode`) and `default_markup`.
