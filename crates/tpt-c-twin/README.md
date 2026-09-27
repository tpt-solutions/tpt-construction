# tpt-c-twin

Live-state layer for digital twins: timestamped sensor readings, asset-to-
sensor mappings, and a `TwinState` snapshot that answers "what is the latest
reading for this sensor?". Reach for it when you need a serializable,
platform-independent model of live telemetry feeding asset records — the kind
of state a building dashboard or twin service would push and query.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `SensorReading`: sensor id, ISO 8601 timestamp, numeric value and unit of
  measure (e.g. `C`, `kPa`, `RPM`).
- `SensorMapping`: ties an `AssetId` to the sensor ids monitoring it, with a
  chainable `add_sensor` builder.
- `TwinState`: an ordered log of readings plus asset mappings.
- `TwinState::push_reading` appends telemetry;
  `TwinState::add_mapping` registers an asset-sensor link.
- `TwinState::latest_for(sensor_id)` returns the most recent reading for a
  sensor by scanning backwards through the log.
- `Default`/`new` empty state, and serde `Serialize`/`Deserialize` on all
  public types for event-stream and storage use.

## Usage

```toml
[dependencies]
tpt-c-twin = "0.1"
```

```rust
use tpt_c_ids::IdFactory;
use tpt_c_twin::{SensorMapping, SensorReading, TwinState};

let mut state = TwinState::new();
state.push_reading(SensorReading::new("T-1", "2026-01-01T00:00:00Z", 22.5, "C"));
state.push_reading(SensorReading::new("T-1", "2026-01-01T01:00:00Z", 23.0, "C"));
state.add_mapping(SensorMapping::new(IdFactory::asset()).add_sensor("T-1"));

let latest = state.latest_for("T-1").unwrap();
assert_eq!(latest.value, 23.0);
```

## Crate relationships

- **Depends on:** `serde`, `tpt-c-model`, `tpt-c-equipment`, `tpt-c-assets`, `tpt-c-fm`, `tpt-c-ids` (id factory).
- **Used by:** Not yet consumed by other workspace crates; see the examples/ directory for integration usage.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
