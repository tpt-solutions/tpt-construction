// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Phase 4 integration example: build a sample project schedule, solve it with
//! the Critical Path Method, project the dates onto a real calendar, run a
//! Monte Carlo risk analysis, and report an earned-value status snapshot.
//!
//! Run with `cargo run -p cpm-schedule`.

use tpt_c_core::ActivityId;
use tpt_c_earned_value::{Amount, EacMethod, EarnedValueStatus};
use tpt_c_ids::IdFactory;
use tpt_c_risk::{
    CostDistribution, DurationDistribution, RiskActivity, RiskModel, WeatherExposure,
};
use tpt_c_schedule::{Activity, Calendar, Dependency, Schedule};
use tpt_c_units::Duration;

/// A small building sequence used for the demonstration.
fn ids() -> (
    ActivityId,
    ActivityId,
    ActivityId,
    ActivityId,
    ActivityId,
    ActivityId,
    ActivityId,
) {
    let f = IdFactory::deterministic;
    (
        ActivityId::from_uuid(f("mobilization")),
        ActivityId::from_uuid(f("excavation")),
        ActivityId::from_uuid(f("foundations")),
        ActivityId::from_uuid(f("frame")),
        ActivityId::from_uuid(f("envelope")),
        ActivityId::from_uuid(f("mep")),
        ActivityId::from_uuid(f("closeout")),
    )
}

/// Working-hour duration for `days` of (8h) work.
fn days(d: f64) -> Duration {
    Duration::from_hours(d * 8.0)
}

/// Build the deterministic CPM schedule aggregate.
pub fn build_schedule() -> Schedule {
    let (a, b, c, d, e, f, g) = ids();
    let cal = Calendar::standard_40h();
    let start = "2026-09-01".parse().expect("valid date");
    let mut sched = Schedule::new("Riverside Outbuilding", cal, start);

    sched
        .add_activity(Activity::new(a, "Mobilization", days(3.0)))
        .unwrap();
    sched
        .add_activity(Activity::new(b, "Excavation", days(5.0)))
        .unwrap();
    sched
        .add_activity(Activity::new(c, "Foundations", days(4.0)))
        .unwrap();
    sched
        .add_activity(Activity::new(d, "Structural Frame", days(10.0)))
        .unwrap();
    sched
        .add_activity(Activity::new(e, "Envelope", days(6.0)))
        .unwrap();
    sched
        .add_activity(Activity::new(f, "MEP Rough-in", days(8.0)))
        .unwrap();
    sched
        .add_activity(Activity::new(g, "Closeout", days(2.0)))
        .unwrap();

    sched
        .add_dependency(Dependency::finish_to_start(a, b))
        .unwrap();
    sched
        .add_dependency(Dependency::finish_to_start(b, c))
        .unwrap();
    sched
        .add_dependency(Dependency::finish_to_start(c, d))
        .unwrap();
    sched
        .add_dependency(Dependency::finish_to_start(d, e))
        .unwrap();
    sched
        .add_dependency(Dependency::finish_to_start(d, f))
        .unwrap();
    sched
        .add_dependency(Dependency::finish_to_start(e, g))
        .unwrap();
    sched
        .add_dependency(Dependency::finish_to_start(f, g))
        .unwrap();

    sched
}

/// Build a risk model mirroring the schedule, with duration and cost uncertainty
/// and weather exposure on excavation.
pub fn build_risk_model() -> RiskModel {
    let (a, b, c, d, e, f, g) = ids();
    let mut model = RiskModel::new();

    let add = |model: &mut RiskModel, id: ActivityId, name: &str, nominal_days: f64, cost: f64| {
        let nominal = nominal_days * 8.0;
        let risk = RiskActivity::new(Activity::new(id, name, days(nominal_days)))
            .with_duration_dist(DurationDistribution::Triangular {
                min_hours: (nominal * 0.8),
                mode_hours: nominal,
                max_hours: (nominal * 1.25),
            })
            .with_cost_dist(CostDistribution::Triangular {
                min: cost * 0.85,
                mode: cost,
                max: cost * 1.2,
            });
        model.add_activity(risk);
    };

    add(&mut model, a, "Mobilization", 3.0, 20_000.0);
    add(&mut model, b, "Excavation", 5.0, 45_000.0);
    add(&mut model, c, "Foundations", 4.0, 60_000.0);
    add(&mut model, d, "Structural Frame", 10.0, 180_000.0);
    add(&mut model, e, "Envelope", 6.0, 90_000.0);
    add(&mut model, f, "MEP Rough-in", 8.0, 120_000.0);
    add(&mut model, g, "Closeout", 2.0, 15_000.0);

    // Weather exposure on excavation: ~3 expected lost working days at 8h/day.
    let excavation = model
        .activities_mut()
        .iter_mut()
        .find(|ra| ra.activity.id == b)
        .unwrap();
    excavation.weather = Some(WeatherExposure {
        expected_lost_days: 3.0,
        hours_per_lost_day: 8.0,
    });

    let (a, b, c, d, e, f, g) = ids();
    model.add_dependency(Dependency::finish_to_start(a, b));
    model.add_dependency(Dependency::finish_to_start(b, c));
    model.add_dependency(Dependency::finish_to_start(c, d));
    model.add_dependency(Dependency::finish_to_start(d, e));
    model.add_dependency(Dependency::finish_to_start(d, f));
    model.add_dependency(Dependency::finish_to_start(e, g));
    model.add_dependency(Dependency::finish_to_start(f, g));

    model
}

fn main() {
    let schedule = build_schedule();
    let result = schedule.compute().expect("schedule solves");
    let project_finish = schedule.project_finish_date(&result);

    println!("=== CPM Schedule: {} ===", schedule.name);
    println!(
        "Project duration: {:.0} working hours (~{:.1} working days)",
        result.project_duration,
        result.project_duration / 8.0
    );
    println!("Forecast finish date: {}\n", project_finish);

    let (a, b, c, d, e, f, g) = ids();
    for id in [a, b, c, d, e, f, g] {
        let s = &result.activities[&id];
        let (start, finish) = schedule.activity_dates(&result, id).unwrap();
        let flag = if s.critical { " *CRITICAL*" } else { "" };
        println!(
            "  {:<18} {:<11} -> {:<11}  float {:>5.1}h{}",
            schedule_name(&schedule, id),
            start,
            finish,
            s.total_float,
            flag
        );
    }
    let critical: Vec<String> = result
        .critical_path
        .iter()
        .map(|id| schedule_name(&schedule, *id))
        .collect();
    println!("\nCritical path: {}", critical.join(" -> "));

    // ---- Risk analysis ---------------------------------------------------
    let risk = build_risk_model();
    let sim = risk.simulate(5000, 20260901).expect("simulation runs");
    println!("\n=== Monte Carlo Risk (5000 trials) ===");
    println!(
        "Schedule (working days): mean {:.1} | P50 {:.1} | P80 {:.1} | P90 {:.1}",
        sim.schedule.mean / 8.0,
        sim.schedule.p50 / 8.0,
        sim.schedule.p80 / 8.0,
        sim.schedule.p90 / 8.0,
    );
    let target_days = result.project_duration / 8.0;
    println!(
        "Probability of meeting the {:.0}-day plan: {:.1}%",
        target_days,
        sim.probability_schedule_met(result.project_duration) * 100.0
    );
    let baseline_cost = sim.cost.mean;
    println!(
        "Total cost (USD): mean {:.0} | P80 {:.0} | P90 {:.0}",
        baseline_cost, sim.cost.p80, sim.cost.p90
    );
    println!(
        "Cost contingency @90%: {:.0} USD",
        sim.cost_contingency(baseline_cost, 0.9)
    );

    // ---- Earned value ----------------------------------------------------
    println!("\n=== Earned Value Status (mid-project) ===");
    let bac = baseline_cost;
    let bcws = bac * 0.60;
    let bcwp = bac * 0.50;
    let acwp = bac * 0.55;
    let ev = EarnedValueStatus::new(
        tpt_c_core::ProjectId::nil(),
        Amount::new(bac, "USD"),
        Amount::new(bcws, "USD"),
        Amount::new(bcwp, "USD"),
        Amount::new(acwp, "USD"),
    )
    .unwrap();
    println!("SPI {:.2} | CPI {:.2}", ev.spi(), ev.cpi());
    println!(
        "EAC (typical) {:.0} | VAC {:.0} | TCPI(bac) {:.2}",
        ev.eac(EacMethod::Typical).value(),
        ev.vac(EacMethod::Typical).value(),
        ev.tcpib()
    );
}

fn schedule_name(schedule: &Schedule, id: ActivityId) -> String {
    schedule
        .activities()
        .get(&id)
        .map(|a| a.name.clone())
        .unwrap_or_else(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_solves_with_critical_path() {
        let sched = build_schedule();
        let r = sched.compute().unwrap();
        assert!(r.project_duration > 0.0);
        assert!(!r.critical_path.is_empty());
        // Frame + envelope/MEP + closeout dominate the critical path.
        assert!(r.activities[&ids().3].critical);
    }

    #[test]
    fn risk_runs_end_to_end() {
        let model = build_risk_model();
        let sim = model.simulate(500, 1).unwrap();
        assert!(sim.schedule.mean > 0.0);
        assert!(sim.schedule.p90 >= sim.schedule.p50);
        assert!(sim.cost.mean > 0.0);
    }
}
