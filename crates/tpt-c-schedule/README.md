# tpt-c-schedule

Construction scheduling: activities linked by precedence relationships, run
through a full Critical Path Method (CPM) forward/backward pass. The crate
models a project schedule as a network of `Activity` nodes with FS/SS/FF/SF
`Dependency` relationships (with lag or lead), projects the engine's internal
working-hour timeline onto real dates via working-time `Calendar`s, supports
activity constraints, and rounds out the schedule lifecycle with `Baseline`
snapshots and progress `Actuals`. Reach for it when a plan needs early/late
dates, float, a critical path, and variance against a captured baseline.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- `ScheduleNetwork`: activities, validated dependencies, and constraints in one schedulable graph; `schedule()` runs the CPM passes
- `Dependency` with `finish_to_start` / `start_to_start` / `finish_to_finish` / `start_to_finish` constructors and `with_lag`
- `ActivityConstraint` with `ConstraintType`: `StartNoEarlierThan`, `StartNoLaterThan`, `FinishNoEarlierThan`, `FinishNoLaterThan`, `MustStartOn`, `MustFinishOn`
- `CpmResult` / `ActivitySchedule`: early/late dates, total and free float, criticality, project duration, and the critical path in topological order
- Cycle detection (`ScheduleError::CycleDetected`), duplicate/dangling validation, empty-schedule guard
- `Schedule` aggregate: network + `Calendar` + project start date, with `compute`, `activity_dates`, and `project_finish_date`
- `Calendar::standard_40h` with `is_working`, `add_working_hours`, and `working_hours_between`; crate-local `NaiveDate`/`Weekday`
- `Baseline` snapshots and `Actuals` with `finish_variance_hours`

## Usage

```toml
[dependencies]
tpt-c-schedule = "0.1"
```

```rust
use tpt_c_core::ActivityId;
use tpt_c_schedule::{Activity, Calendar, Dependency, Schedule};
use tpt_c_units::Duration;

let a = ActivityId::from_uuid(uuid::Uuid::now_v7());
let b = ActivityId::from_uuid(uuid::Uuid::now_v7());

let cal = Calendar::standard_40h();
let start = "2026-09-01".parse().expect("valid date");
let mut sched = Schedule::new("Demo", cal, start);

sched.add_activity(Activity::new(a, "Excavation", Duration::from_hours(40.0)))?;
sched.add_activity(Activity::new(b, "Foundations", Duration::from_hours(32.0)))?;
sched.add_dependency(Dependency::finish_to_start(a, b))?;

let result = sched.compute()?;
assert!(!result.critical_path.is_empty());
let (start, finish) = sched.activity_dates(&result, b)?;
let finish_date = sched.project_finish_date(&result);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `tpt-c-core`, `tpt-c-units` (dev: `tpt-c-ids`, `uuid`)
- **Used by:** `tpt-c-risk` (Monte Carlo over the CPM engine), `tpt-c-wasm` (WASM bindings), the `cpm-schedule` example, and the `tpt` CLI's dashboard server (`examples/tpt`)

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
