# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `PropertyValue` (`Text`, `Number`, `Boolean`), `Property`, and
  `PropertySet` with `new` and `with`
- `Quantity` enum (`Length`, `Area`, `Volume`, `Count`, `Mass`, `Duration`)
  with `base_value`, plus `NamedQuantity` and `QuantitySet` with `new`
  and `with`
- `MaterialLayer` for layered element build-ups
- `Element` with builder methods `in_storey`, `classified`,
  `with_external_id`, `with_layer`, `with_property_set`,
  `with_quantity_set`; implements `Identified`
- Spatial and grouping containers: `Assembly`, `Zone`, `System`, `Storey`,
  `Building`, `Site`
- `Project` aggregate with `new`, `with_default_system`, `add_element`,
  `element`, `elements_of_category`, and `element_count`
