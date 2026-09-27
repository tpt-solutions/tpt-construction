# 5. Solve a CPM schedule

> **Crates used:** `tpt-c-schedule`, `tpt-c-ids`, `tpt-c-units`, `tpt-c-core`

The Critical Path Method engine runs forward/backward passes over a network
of activities and finish-to-start (also SS/FF/SF) dependencies and reports
early/late dates, total and free float, and the critical path — all in
working hours, so calendar projection stays a separate, explicit step.

## Cargo.toml

```toml
[dependencies]
tpt-c-core = "0.1"
tpt-c-ids = "0.1"
tpt-c-schedule = "0.1"
tpt-c-units = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use tpt_c_core::ActivityId;
use tpt_c_ids::IdFactory;
use tpt_c_schedule::{Activity, Dependency, ScheduleNetwork};
use tpt_c_units::Duration;

/// Deterministic activity id from a name (same name -> same UUID).
fn id(name: &str) -> ActivityId {
    ActivityId::from_uuid(IdFactory::deterministic(name))
}

/// A working-day duration (8 h days, matching the default calendar).
fn days(d: f64) -> Duration {
    Duration::from_hours(d * 8.0)
}

fn main() {
    let plan: [(&str, f64); 4] = [
        ("dig", 2.0),
        ("foundations", 3.0),
        ("frame", 6.0),
        ("closeout", 1.0),
    ];

    let mut net = ScheduleNetwork::new();
    for (name, duration) in plan {
        // Duplicate ids are rejected with DuplicateActivity.
        net.add_activity(Activity::new(id(name), name, days(duration)))
            .unwrap();
    }
    // A linear chain; finish-to-start is the common case. Dependencies that
    // reference unknown activities, or cycles, are errors — not panics.
    for (from, to) in [
        ("dig", "foundations"),
        ("foundations", "frame"),
        ("frame", "closeout"),
    ] {
        net.add_dependency(Dependency::finish_to_start(id(from), id(to)))
            .unwrap();
    }

    // Forward + backward pass.
    let result = net.schedule().unwrap();
    println!(
        "project duration: {:.0} working hours ({:.0} days)",
        result.project_duration,
        result.project_duration / 8.0,
    );

    // Per-activity results live in a map; walk the plan order for stable
    // output. Float is how long an activity can slip without moving the
    // project finish (total) or its successors (free).
    for (name, _) in plan {
        let s = &result.activities[&id(name)];
        println!(
            "{:<12} float {:>5.1} h {}",
            name,
            s.total_float,
            if s.critical { "* critical" } else { "" },
        );
    }

    let path: Vec<String> = result
        .critical_path
        .iter()
        .map(|a| result.activities[a].critical.to_string())
        .collect();
    let _ = path; // (the path itself is `result.critical_path`, in topological order)
}
```

## Run it

```bash
cargo run
```

```text
project duration: 96 working hours (12 days)
dig           float   0.0 h * critical
foundations   float   0.0 h * critical
frame         float   0.0 h * critical
closeout      float   0.0 h * critical
```

A linear chain is all-critical; give one activity float by adding a parallel
predecessor and watch the numbers separate.

## How it works

1. `ScheduleNetwork` is the pure CPM structure — activities keyed by id,
   plus dependency and constraint lists. It serializes to JSON (this is also
   the body format of `tpt serve`'s `/api/v1/schedule` endpoint).
2. `schedule()` runs both passes and returns `CpmResult` with per-activity
   `ActivitySchedule` (early/late start/finish, floats, criticality), the
   project duration, and the critical path in topological order.
3. Everything is in **working hours**. To project wall-clock dates, use the
   calendar-aware `Schedule` aggregate (`Calendar::standard_40h`,
   `activity_dates`, `project_finish_date`) — see
   `examples/cpm-schedule/src/main.rs` for that flow, including baselines.
