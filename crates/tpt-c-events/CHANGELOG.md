# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `DomainEvent` enum with nine variants (`ModelImported`, `QuantityAdjusted`,
  `CostItemApplied`, `EstimateApproved`, `ScheduleUpdated`, `RFICreated`,
  `RFIAnswered`, `SubmittalApproved`, `PaymentApplicationSubmitted`) and a
  stable `kind()` name per variant.
- `StoredEvent<E>` envelope (1-based `seq`, `event_id`, `occurred_at`, `actor`,
  payload) and `EventStore<E>` append-only log with `append`, `len`, `is_empty`,
  `version`, `get`, `all`, `iter_from`, `to_json`, and `from_json`.
- `Projection<E>` trait plus `replay` and `replay_from` free functions for full
  and incremental rebuilds of read models.
- Built-in projections: `EventTypeCounter` (per-kind counts) and `AuditTimeline`
  (chronological `AuditLine` records).
- `AuditTrail` convenience alias for `EventStore<DomainEvent>`.
