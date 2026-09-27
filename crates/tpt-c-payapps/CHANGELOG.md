# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `ScheduleOfValues` with `SovItem` line items and `new`, `add_item`, `total`,
  `len`, `is_empty` (empty SOV totals to `Money::zero("USD")`).
- `PaymentApplication` for period draw-downs: `new` (seeds zero amounts in the
  SOV's currency), `with_sov`, `set_work_completed`, `set_stored_materials`,
  `set_retainage` (fraction validated to `[0, 1]`), `set_previously_certified`,
  `gross_val`, `retainage_amount`, and `net_claim`.
- `PaymentApplicationStatus` lifecycle (`Draft`, `Submitted`, `UnderReview`,
  `Approved`, `Rejected`, `Certified`) with `submit`, `begin_review`, `approve`,
  `reject`, and `certify` (only from `Approved`).
- `PayAppError` (`CurrencyMismatch`, `OutOfRange`) with `From<tpt_c_cost::CostError>`.
