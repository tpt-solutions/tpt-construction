# tpt-c-classification

Construction classification systems and project-level mapping. Provides
first-class support for the major classification taxonomies used in
construction — CSI MasterFormat, ASTM UniFormat, OmniClass, and Uniclass —
plus a `Custom` scheme, canonical division/group tables with title lookup,
and a bi-directional map so a project's own category labels (e.g.
`foundation-slab`) can resolve to recognized codes (e.g. MasterFormat
`03 30 00`) and back.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `ClassificationSystem` enum: `MasterFormat`, `UniFormat`, `OmniClass`,
  `Uniclass`, `Custom` (snake_case serde representation)
- `Classification` value combining system, code, and optional title, with
  `new` and `with_title` builders
- `masterformat` module: canonical `DIVISIONS` table (00–49 with titles) and
  `title_for` lookup by two-digit code
- `uniformat` module: canonical `GROUPS` table (A–H) and `title_for` lookup
  by letter
- `ProjectClassificationMap`: bi-directional project-category↔classification
  map with `insert`, `classify`, and `category_for`
- Pure, serializable data — only `serde` as a dependency

## Usage

```toml
[dependencies]
tpt-c-classification = "0.1"
```

```rust
use tpt_c_classification::{
    masterformat, uniformat, Classification, ClassificationSystem,
    ProjectClassificationMap,
};

// Canonical taxonomy lookups.
assert_eq!(masterformat::title_for("03"), Some("Concrete"));
assert_eq!(uniformat::title_for("A"), Some("Substructure"));

// Map a project category to MasterFormat and back.
let mut m = ProjectClassificationMap::new();
m.insert(
    "foundation-slab",
    Classification::new(ClassificationSystem::MasterFormat, "03 30 00")
        .with_title("Cast-in-Place Concrete"),
);
assert_eq!(m.classify("foundation-slab").unwrap().code, "03 30 00");
assert_eq!(
    m.category_for(ClassificationSystem::MasterFormat, "03 30 00"),
    Some(&"foundation-slab".to_string())
);
```

## Crate relationships

- **Depends on:** `serde` only (no workspace dependencies)
- **Used by:** `tpt-c-model`, `tpt-c-ifc`, `tpt-c-cost`, `tpt-c-estimating`,
  and `tpt-c-quantities`

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
