# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `read_las(&[u8])` — parse a LAS public header block and point records
  (point data record formats 0–3) into `(LasHeader, Vec<LasPoint>)`
- `LasHeader` — version, offset to points, point format/record length, point
  count, scale/offset factors, and min/max real-world bounds
- `LasPoint` — real-world x/y/z, intensity, return number, number of returns,
  ASPRS classification code
- `classification` module of ASPRS code constants (`GROUND`, `BUILDING`,
  vegetation bands, `WATER`, ...)
- `approximate_volume(&LasHeader)` — bounding-box volume estimate
- `LasError` — `BadSignature`, `UnsupportedFormat(u8)`, `Truncated`,
  `InvalidNumber`
