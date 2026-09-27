# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `Crs` coordinate reference system with `WGS84` and `WEB_MERCATOR`
  constants and `from_epsg`
- `GeographicCoordinate` (latitude/longitude degrees, elevation metres) and
  `LocalGridCoordinate` (east/north/elevation)
- `Datum` enum (`Wgs84`, `Nad83`, `Osgb36`, `Local`)
- `SiteDatum` with `new`, `with_rotation`, `to_local`, and `to_geo`
  tangent-plane transforms between local grid and geographic coordinates
- `GridLine` survey grid-line definition for staking
