# tpt-c-field

Field execution records: the day-to-day reality of a jobsite. The `DailyLog`
aggregate captures one project day — the `WeatherRecord` that shaped it, the
`ManpowerRecord`s that performed the work, free-form `WorkRecord` entries with
optional quantities, and `SiteObservation`s (safety / quality / environmental)
with a review-and-resolve lifecycle. Daily logs can link the RFIs they spawned
via `tpt-c-ids::RFIId`, tying field capture to the approval workflow. Roll-ups
like `total_man_hours`, `trade_count`, and `open_observations` make the log
reportable. Reach for it when site reality needs to be recorded, audited, and
connected to downstream workflows.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `DailyLog::new(project_id, date, created_by)` with `set_weather`, `add_manpower`, `add_work`, `add_observation`, and `link_rfi`
- Instant roll-ups: `total_man_hours`, `trade_count`, and `open_observations`
- `WeatherRecord` (condition, temps, precipitation, wind) with `is_inclement` wash-out detection across `WeatherCondition`s
- `ManpowerRecord` (trade, headcount, hours) with `total_hours`
- `WorkRecord` builder: `with_location` and `with_quantity(quantity, unit)`
- `SiteObservation` lifecycle: `ObservationCategory` (Safety/Quality/Environmental/General), `Severity` (Low→Critical), `ObservationStatus` (Open→InReview→Closed) with `review`/`resolve` audit stamps
- Crate-local `DailyLogId` / `WorkRecordId` UUID newtypes with `Display`/`FromStr` round-trip, plus `ObservationId`

## Usage

```toml
[dependencies]
tpt-c-field = "0.1"
```

```rust
use tpt_c_core::ProjectId;
use tpt_c_field::{
    DailyLog, ObservationCategory, Severity, SiteObservation, WeatherCondition, WeatherRecord,
    WorkRecord,
};

let mut log = DailyLog::new(ProjectId::nil(), "2026-03-01", "foreman")
    .set_weather(WeatherRecord::new(WeatherCondition::Clear, 18.0, 6.0));
log.add_manpower("carpenters", 4, 8.0);
log.add_manpower("electricians", 2, 6.5);
log.add_work(
    WorkRecord::new("Pour slab")
        .with_location("Grid A1-C3")
        .with_quantity(12.5, "m3"),
);
log.add_observation(SiteObservation::new(
    ObservationCategory::Safety,
    "Unguarded floor opening",
    Severity::High,
));
assert_eq!(log.trade_count(), 2);
assert_eq!(log.total_man_hours(), 4.0 * 8.0 + 2.0 * 6.5);
assert_eq!(log.open_observations(), 1);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core`, `tpt-c-ids`
- **Used by:** the `field-execution-e2e` example (daily log → RFI → document → pay-app chain)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
