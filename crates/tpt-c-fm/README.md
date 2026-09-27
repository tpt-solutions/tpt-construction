# tpt-c-fm

Facility-management domain types: COBie-style handover records and facility
asset registers with the space/system taxonomy used when a building is handed
over to its operator. Each record states which system an item belongs to
(e.g. HVAC, electrical) and which space it is installed in, with optional
manufacturer, model and serial metadata. Reach for it when modelling handover
or asset-register data exchanges in the COBie style.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `FacilityAsset`: handover asset-register entry keyed by the workspace's
  stable `AssetId`, with `name`, `system`, `space` and optional
  `manufacturer`/`model`/`serial_number`.
- Builder methods `FacilityAsset::with_manufacturer` and `with_serial`.
- `CobieComponent`: COBie-style component record (`name`, `description`,
  `space`, `system`, optional `manufacturer` and `model_number`).
- All types derive `Serialize`/`Deserialize` and round-trip through JSON;
  optional fields are skipped when absent.
- Identifiers come from the shared `tpt-c-core` crate, so FM records compose
  with the rest of the workspace.

## Usage

```toml
[dependencies]
tpt-c-fm = "0.1"
```

```rust
use tpt_c_fm::{CobieComponent, FacilityAsset};
use tpt_c_ids::IdFactory;

let asset = FacilityAsset::new(IdFactory::asset(), "AHU-1", "HVAC", "Level 1")
    .with_manufacturer("Trane", "S Helix")
    .with_serial("SN12345");
assert_eq!(asset.system, "HVAC");
assert_eq!(asset.space, "Level 1");

let comp = CobieComponent::new("VAV-1", "Variable air volume box", "Room 101", "HVAC");
// Serializes to JSON (serde) and back losslessly.
```

## Crate relationships

- **Depends on:** `serde`, `serde_json`, `tpt-c-core` (`AssetId`), `tpt-c-ids` (test id factory), `tpt-c-model`.
- **Used by:** `tpt-c-twin` (declares `tpt-c-fm` as a workspace dependency).

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
