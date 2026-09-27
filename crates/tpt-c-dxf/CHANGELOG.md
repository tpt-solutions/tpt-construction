# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `DxfDocument::parse(&str)` — tokenize ASCII DXF group-code/value pairs and
  collect entities from the ENTITIES section
- `DxfEntity` enum: `Line`, `LwPolyline` (vertices + closed flag), `Circle`,
  `Point`, `Text`, and `Other` (unknown types preserved by name)
- `DxfDocument::segments()` — flatten entities to start/end segment pairs
  (polyline edges including closing edge, 24-segment circle approximations)
- `DxfDocument::len()` / `DxfDocument::is_empty()` entity counts
- `DxfError` — `NoEntities`, `BadGroupCode`, `UnexpectedEof`
