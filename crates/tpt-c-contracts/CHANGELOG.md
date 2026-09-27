# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Contract` aggregate: `add_party`, `add_responsibility`, `add_item`, and
  `total_value` (currency-checked sum), with `Party`, `Responsibility`, and
  `ContractItem` support types.
- `Amount` (value + ISO 4217 currency code) with `checked_add` currency guard.
- `Notice` driven by `tpt_c_workflow::NoticeWorkflow` with `state`,
  `acknowledge`, and `resolve` convenience methods.
- `Claim` with `ClaimKind` (`Delay`, `Disruption`, `Acceleration`, `Payment`,
  `Other`), optional claimed `Amount`, `ClaimStatus` lifecycle (`submit`,
  `resolve`).
- `ComplianceEvent` with `ComplianceStatus` (`Open`, `Satisfied`, `Overdue`) and
  `satisfy` / `mark_overdue` transitions.
- `ContractError` (`NotFound`, `OutOfRange`, core error passthrough).
