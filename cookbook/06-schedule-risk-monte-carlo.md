# 6. Schedule risk with Monte Carlo

> **Crates used:** `tpt-c-risk`, `tpt-c-schedule`, `tpt-c-ids`, `tpt-c-units`, `tpt-c-core`

Turn a deterministic plan into a probabilistic one: attach duration and cost
distributions to activities, add weather exposure, then simulate thousands
of runs to get percentile outcomes, the probability of hitting a date, and a
defensible cost contingency. The simulator is fully deterministic given a
seed — reruns reproduce the exact numbers.

## Cargo.toml

```toml
[dependencies]
tpt-c-core = "0.1"
tpt-c-ids = "0.1"
tpt-c-risk = "0.1"
tpt-c-schedule = "0.1"
tpt-c-units = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

use tpt_c_core::ActivityId;
use tpt_c_ids::IdFactory;
use tpt_c_risk::{
    CostDistribution, DurationDistribution, RiskActivity, RiskModel, WeatherExposure,
};
use tpt_c_schedule::{Activity, Dependency};
use tpt_c_units::Duration;

fn id(name: &str) -> ActivityId {
    ActivityId::from_uuid(IdFactory::deterministic(name))
}

fn main() {
    let mut model = RiskModel::new();

    // One risk activity wraps a normal schedule activity plus uncertainty.
    let add = |model: &mut RiskModel, name: &str, nominal_days: f64, cost: f64| {
        let nominal_hours = nominal_days * 8.0;
        let risk = RiskActivity::new(Activity::new(id(name), name, Duration::from_hours(nominal_hours)))
            .with_duration_dist(DurationDistribution::Triangular {
                min_hours: nominal_hours * 0.8,
                mode_hours: nominal_hours,
                max_hours: nominal_hours * 1.25,
            })
            .with_cost_dist(CostDistribution::Triangular {
                min: cost * 0.85,
                mode: cost,
                max: cost * 1.2,
            });
        model.add_activity(risk);
    };

    add(&mut model, "dig", 2.0, 18_000.0);
    add(&mut model, "foundations", 3.0, 42_000.0);

    // Weather exposure: ~2 expected lost 8-hour days on foundations.
    let activities = model.activities_mut();
    if let Some(f) = activities.iter_mut().find(|r| r.activity.name == "foundations") {
        f.weather = Some(WeatherExposure {
            expected_lost_days: 2.0,
            hours_per_lost_day: 8.0,
        });
    }

    // Chain the work: dig -> foundations.
    model.add_dependency(Dependency::finish_to_start(id("dig"), id("foundations")));

    // 10,000 trials from a fixed seed => fully reproducible statistics.
    let sim = model.simulate(10_000, 42).unwrap();

    // The deterministic plan is 40 working hours (5 days).
    let plan_hours = 40.0;
    println!(
        "schedule (working days): mean {:.1} | P50 {:.1} | P80 {:.1} | P90 {:.1}",
        sim.schedule.mean / 8.0,
        sim.schedule.p50 / 8.0,
        sim.schedule.p80 / 8.0,
        sim.schedule.p90 / 8.0,
    );
    println!(
        "probability of meeting the {:.0}-day plan: {:.1}%",
        plan_hours / 8.0,
        sim.probability_schedule_met(plan_hours) * 100.0,
    );
    println!(
        "cost (USD): mean {:.0} | P80 {:.0} | P90 {:.0}",
        sim.cost.mean, sim.cost.p80, sim.cost.p90,
    );
    println!(
        "cost contingency @90%: {:.0} USD",
        sim.cost_contingency(sim.cost.mean, 0.9),
    );
}
```

## Run it

```bash
cargo run
```

Output is stable for seed `42` (values shown from the verified run of this
recipe; rerun to confirm):

```text
schedule (working days): mean 7.1 | P50 6.9 | P80 8.2 | P90 9.1
probability of meeting the 5-day plan: 5.5%
cost (USD): mean 60988 | P80 63847 | P90 65450
cost contingency @90%: 4462 USD
```

## How it works

1. `RiskActivity` pairs the deterministic `Activity` with uncertainty:
   `DurationDistribution` and `CostDistribution` support Triangular (and
   other) shapes; `WeatherExposure` samples lost days as a Poisson process
   and converts them into extra hours.
2. `RiskModel` mirrors the network (`add_activity`, `add_dependency`,
   `add_constraint`) and CPM-solves every trial internally.
3. `simulate(iterations, seed)` returns `SimulationResult` with schedule and
   cost `Statistics` (`mean`, `p50`, `p80`, `p90`), plus helpers:
   `probability_schedule_met(target_hours)`, `probability_cost_met`, and
   `cost_contingency(baseline, confidence)`.
4. Same seed, same numbers — risk results you can put in a basis-of-estimate
   document and defend in review.
