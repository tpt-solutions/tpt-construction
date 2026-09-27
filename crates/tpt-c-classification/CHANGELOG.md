# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `ClassificationSystem` enum (`MasterFormat`, `UniFormat`, `OmniClass`,
  `Uniclass`, `Custom`)
- `Classification` value with `new` and `with_title`
- `masterformat` module with the canonical `DIVISIONS` table (00–49) and
  `title_for` lookup
- `uniformat` module with the canonical `GROUPS` table (A–H) and
  `title_for` lookup
- `ProjectClassificationMap` with `new`, `insert`, `classify`, and
  `category_for` for bi-directional project-category↔classification mapping
