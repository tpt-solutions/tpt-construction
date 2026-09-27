# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `export(&Project) -> GltfDocument` glTF 2.0 scene builder (one node per model
  element plus a project root node)
- `GltfDocument::to_json()` — pretty-printed, standards-compliant glTF 2.0 JSON
- glTF struct model: `Asset`, `Scene`, `Node`, `Mesh`, `Primitive`, `Accessor`,
  `BufferView`, `Buffer`
- Shared unit-cube mesh (8 float positions, 36 `u16` indices) in a single base64
  data-URI buffer
- Element dimensions from `Length`/`Height`/`Width` property-set values with a
  unit-cube fallback
- Element metadata (category, storey, external id, classification) mirrored into
  node `extras`
- Hand-rolled base64 encoder (no external base64 dependency)
