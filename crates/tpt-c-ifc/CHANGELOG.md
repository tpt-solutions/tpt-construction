# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `parse` for ISO-10303-21 (STEP) physical files, returning a `StepDoc`
- `StepValue` enum (`Ref`, `Str`, `Real`, `Int`, `Enum`, `List`,
  `Unspecified`, `Star`) covering the STEP parameter grammar
- `StepEntity` with `id`, `ty`, `params` and the `str_at`, `real_at`,
  `int_at`, `ref_at`, `list_at` accessors
- `StepDoc` with `get`, `by_type`, and `by_type_prefix` lookups
- `to_model` mapping a `StepDoc` to a `tpt_c_model::Project`: elements for
  `IfcWall`/`IfcSlab`/`IfcColumn`/`IfcBeam`/`IfcDoor`/`IfcWindow`/
  `IfcSpace` with deterministic ids from IFC GUIDs, storey containment via
  `IfcRelContainedInSpatialStructure`, property sets via
  `IfcRelDefinesByProperties`/`IfcPropertySingleValue`, and quantity sets
  via `IfcElementQuantity` (`IfcQuantityLength`, `IfcQuantityArea`,
  `IfcQuantityVolume`, `IfcQuantityCount`, `IfcQuantityWeight`)
- `IfcError` (`InvalidHeader`, `Parse`, `DanglingRef`, `MissingProject`)
