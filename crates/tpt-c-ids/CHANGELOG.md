# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `TPT_NAMESPACE` constant for UUIDv5 derivation, distinct from the
  RFC 4122 standard namespaces
- `IdFactory` with `uuid7` (time-ordered UUIDv7), `deterministic`
  (reproducible UUIDv5 from a name), `element`, `deterministic_element`,
  `asset`, and `estimate` constructors
- `ExternalId` enum (`IfcGuid`, `RevitId`, `CostCode`, `AssetTag`, `Other`)
  with tagged serde representation and a `value` accessor
- `ExternalIdMap<I>` bi-directional mapping with `new`, `insert`,
  `externals_for`, and `resolve`
- `ExternalIdConflict` error returned when an external id is reused for a
  different internal id
- Re-export of the `tpt-c-core` id newtypes (`ProjectId`, `ElementId`,
  `AssetId`, ...)
