# tpt-c-assets

Asset lifecycle tracking for physical building equipment: commissioning
through operation to replacement, including warranty windows, serial numbers
and installation dates. Reach for it when you need a serializable asset record
with a lifecycle status and the ability to mark an asset as replaced by
another, forming a replaceable asset chain.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `Asset` record keyed by the workspace's stable `AssetId`, with `name`,
  optional `serial_number`, `warranty_expiry` and `install_date`
  (ISO 8601 strings), and a `replaced_by` link.
- `AssetStatus` lifecycle enum: `Active`, `Standby`, `UnderMaintenance`,
  `Retired`, `Replaced` (serialized as snake_case).
- Builder methods `with_serial`, `with_warranty` and `with_install_date`.
- `Asset::replace(new_id)` marks the asset `Replaced` and records its
  successor.
- `Asset::is_active()` reports whether the asset is still in service
  (`Active` or `Standby`).
- Serde `Serialize`/`Deserialize` with optional fields skipped when absent.

## Usage

```toml
[dependencies]
tpt-c-assets = "0.1"
```

```rust
use tpt_c_assets::{Asset, AssetStatus};
use tpt_c_ids::IdFactory;

let id = IdFactory::asset();
let asset = Asset::new(id, "Chiller-1")
    .with_serial("SN999")
    .with_warranty("2028-06-01")
    .with_install_date("2026-01-15");
assert!(asset.is_active());

let new_id = IdFactory::asset();
let retired = asset.replace(new_id);
assert_eq!(retired.status, AssetStatus::Replaced);
assert_eq!(retired.replaced_by, Some(new_id));
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-core` (`AssetId`), `tpt-c-ids` (test id factory).
- **Used by:** `tpt-c-maintenance` and `tpt-c-twin` (both declare `tpt-c-assets` as a workspace dependency).

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
