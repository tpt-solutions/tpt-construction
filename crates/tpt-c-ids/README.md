# tpt-c-ids

Deterministic and standard identifier generation plus external-ID mapping.
Internal domain entities are addressed by the UUID-backed identifiers from
`tpt-c-core`; interoperating with authoring tools requires mapping those
internal ids to external identifiers such as IFC GUIDs, Revit element ids,
cost codes, and asset tags. This crate is that bridge: it offers time-ordered
UUIDv7 generation, reproducible UUIDv5 derivation, and a typed
bi-directional internal/external id map.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `IdFactory` with `uuid7()` (time-ordered, random) and `deterministic(name)`
  (stable UUIDv5 derivation — same name always yields the same UUID)
- Convenience constructors `element()`, `deterministic_element(name)`,
  `asset()`, and `estimate()` returning typed ids
- `TPT_NAMESPACE`, a fixed UUIDv5 namespace kept distinct from the RFC 4122
  DNS/URL/OID/IETF namespaces so TPT-derived ids never collide with
  vendor-generated ones
- `ExternalId` enum covering `IfcGuid`, `RevitId`, `CostCode`, `AssetTag`,
  and `Other`, with tagged serde representation and a `value()` accessor
- `ExternalIdMap<I>`: bi-directional internal↔external mapping where one
  internal id may carry several external aliases; `insert`, `externals_for`,
  and `resolve`
- `ExternalIdConflict` error reporting the external id and the internal id
  that already owns it
- Re-exports the `tpt-c-core` id newtypes for convenience

## Usage

```toml
[dependencies]
tpt-c-ids = "0.1"
```

```rust
use tpt_c_ids::{ElementId, ExternalId, ExternalIdMap, IdFactory};

let mut map: ExternalIdMap<ElementId> = ExternalIdMap::new();
let eid = IdFactory::deterministic_element("w1");
map.insert(eid, ExternalId::IfcGuid("3O0MLfx3BDgvhRaydMkJh6".into()))
    .unwrap();
map.insert(eid, ExternalId::RevitId("123456".into())).unwrap();

assert_eq!(map.externals_for(&eid).len(), 2);
let guid = ExternalId::IfcGuid("3O0MLfx3BDgvhRaydMkJh6".into());
assert_eq!(map.resolve(&guid), Some(&eid));

// Deterministic derivation is reproducible across runs.
assert_eq!(
    IdFactory::deterministic("wall-1"),
    IdFactory::deterministic("wall-1")
);
```

## Crate relationships

- **Depends on:** `serde`, `uuid`, and `tpt-c-core` (for the id newtypes it
  re-exports and wraps)
- **Used by:** `tpt-c-model`, `tpt-c-ifc`, `tpt-c-gltf`, `tpt-c-estimating`,
  `tpt-c-cost`, `tpt-c-contracts`, `tpt-c-documents`, `tpt-c-workflow`,
  `tpt-c-field`, `tpt-c-fm`, `tpt-c-payapps`, `tpt-c-quantities`,
  `tpt-c-safety`, `tpt-c-twin`, `tpt-c-events`, `tpt-c-equipment`,
  `tpt-c-maintenance`, `tpt-c-assets`, `tpt-c-change`, and `tpt-c-api`, plus
  the `cpm-schedule` and `field-*` examples

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
