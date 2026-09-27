# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `QuantityKind` (Count/Length/Area/Volume/Mass) and `MeasuredQuantity` with
  `kind()`, `base_value()`, `apply_waste()`, and `from_model()` conversion from
  `tpt_c_model::Quantity`
- `TakeoffQuantity` — named net quantity with waste factor and `manual` flag,
  plus `gross()`
- `TakeoffEngine` — `new()` / `with_rules()` / `run(&Project)` per-element
  extraction over `TakeoffItem`s
- `TakeoffResult` — `total_net(kind)`, `total_gross(kind)`, and
  `override_quantity(element_id, name, value)` manual estimator overrides
- `QuantityRules` — per-category waste overrides (`with_*_waste`) and lookups
  (`concrete_waste`, `formwork_waste`, `rebar_waste`, `paint_waste`,
  `flooring_waste`, `waste_for_kind`)
- Derived rule functions: `concrete_volume`, `formwork_area`,
  `rebar_weight_for`, `paint_area`, `flooring_area_with_deductions`
- Default waste constants: `DEFAULT_WASTE_CONCRETE`, `DEFAULT_WASTE_FORMWORK`,
  `DEFAULT_WASTE_REBAR`, `DEFAULT_WASTE_PAINT`, `DEFAULT_WASTE_FLOORING`
- `TakeoffError::MissingQuantity`
