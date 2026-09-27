# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `DailyLog` aggregate: `new`, `set_weather`, `add_manpower`, `add_work`, `add_observation`, `link_rfi`, `total_man_hours`, `trade_count`, and `open_observations`.
- `WorkRecord` with `new`, `with_location`, and `with_quantity`.
- `WeatherRecord` with `new` and `is_inclement`, over the `WeatherCondition` enum (Clear/Cloudy/Rain/Snow/Wind/Storm).
- `ManpowerRecord` (trade, headcount, hours) with `total_hours`.
- `SiteObservation` with `new`, `review`, and `resolve`, plus the `ObservationCategory`, `Severity`, and `ObservationStatus` enums and the `ObservationId` newtype.
- Crate-local `DailyLogId` and `WorkRecordId` UUID identifier newtypes (`from_uuid`, `nil`, `as_uuid`, `Display`, `FromStr`).
- `FieldError` (`MissingRequired`, `OutOfRange`, `Core`) for record validation.
