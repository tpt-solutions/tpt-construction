# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- UUID-backed identifier newtypes: `ProjectId`, `ContractId`, `ModelId`,
  `ElementId`, `EstimateId`, `ScheduleId`, `ActivityId`, `RFIId`,
  `SubmittalId`, `ChangeOrderId`, `AssetId`, `ClaimId`, `ContractItemId`,
  `DocumentId`, `IssueId`, `NoticeId`, `PaymentApplicationId`,
  `PunchListId`, `SafetyIncidentId`, `TransmittalId` — each with
  `from_uuid`, `nil`, `as_uuid`, `Display`, and `FromStr`
- `CoreError` error enum (`InvalidId`, `MissingRequired`, `InvariantViolated`,
  `NotFound`, `OutOfRange`, `IllegalState`) and the `Result<T>` alias
- `AuditMeta` with `new` and `with_note` for actor/timestamp/note provenance
- `ProjectRef` lightweight project reference
- `ProjectContext` with `new`, `with_id`, `with_organization`,
  `with_currency` (currency defaults to `USD`); implements `ProjectScoped`
- `ProjectScoped` and `Identified` capability traits
