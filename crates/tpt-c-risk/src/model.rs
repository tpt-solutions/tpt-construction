// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Monte Carlo risk model for schedules and costs.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tpt_c_core::ActivityId;
use tpt_c_schedule::{Activity, ActivityConstraint, Dependency, ScheduleError, ScheduleNetwork};
use tpt_c_units::Duration;

use crate::prob::Rng;

/// Errors raised while building or running a risk simulation.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RiskError {
    /// The risk model has no activities to simulate.
    #[error("risk model has no activities")]
    EmptyModel,
    /// A simulation was requested with zero iterations.
    #[error("simulation requires at least one iteration")]
    NoIterations,
    /// The underlying schedule could not be solved.
    #[error("schedule error: {0}")]
    Schedule(#[from] ScheduleError),
}

/// A sampling distribution for an activity's duration (working hours).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum DurationDistribution {
    /// Bounded uniform duration.
    Uniform {
        /// Minimum duration in working hours.
        min_hours: f64,
        /// Maximum duration in working hours.
        max_hours: f64,
    },
    /// Triangular duration with a most-likely mode.
    Triangular {
        /// Minimum duration in working hours.
        min_hours: f64,
        /// Most-likely duration in working hours.
        mode_hours: f64,
        /// Maximum duration in working hours.
        max_hours: f64,
    },
    /// Normal (Gaussian) duration.
    Normal {
        /// Mean duration in working hours.
        mean_hours: f64,
        /// Standard deviation in working hours.
        sd_hours: f64,
    },
    /// PERT (Beta-approximated) duration.
    Pert {
        /// Minimum duration in working hours.
        min_hours: f64,
        /// Most-likely duration in working hours.
        mode_hours: f64,
        /// Maximum duration in working hours.
        max_hours: f64,
    },
}

impl DurationDistribution {
    /// Draw a non-negative duration sample.
    pub fn sample(&self, rng: &mut Rng) -> f64 {
        let v = match self {
            DurationDistribution::Uniform {
                min_hours,
                max_hours,
            } => rng.uniform(*min_hours, *max_hours),
            DurationDistribution::Triangular {
                min_hours,
                mode_hours,
                max_hours,
            } => rng.triangular(*min_hours, *mode_hours, *max_hours),
            DurationDistribution::Normal {
                mean_hours,
                sd_hours,
            } => rng.normal(*mean_hours, *sd_hours),
            DurationDistribution::Pert {
                min_hours,
                mode_hours,
                max_hours,
            } => rng.pert(*min_hours, *mode_hours, *max_hours),
        };
        v.max(0.0)
    }
}

/// A sampling distribution for an activity's cost (project currency).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CostDistribution {
    /// Bounded uniform cost.
    Uniform {
        /// Minimum cost.
        min: f64,
        /// Maximum cost.
        max: f64,
    },
    /// Triangular cost with a most-likely mode.
    Triangular {
        /// Minimum cost.
        min: f64,
        /// Most-likely cost.
        mode: f64,
        /// Maximum cost.
        max: f64,
    },
    /// Normal (Gaussian) cost.
    Normal {
        /// Mean cost.
        mean: f64,
        /// Standard deviation.
        sd: f64,
    },
    /// PERT (Beta-approximated) cost.
    Pert {
        /// Minimum cost.
        min: f64,
        /// Most-likely cost.
        mode: f64,
        /// Maximum cost.
        max: f64,
    },
}

impl CostDistribution {
    /// Draw a non-negative cost sample.
    pub fn sample(&self, rng: &mut Rng) -> f64 {
        let v = match self {
            CostDistribution::Uniform { min, max } => rng.uniform(*min, *max),
            CostDistribution::Triangular { min, mode, max } => rng.triangular(*min, *mode, *max),
            CostDistribution::Normal { mean, sd } => rng.normal(*mean, *sd),
            CostDistribution::Pert { min, mode, max } => rng.pert(*min, *mode, *max),
        };
        v.max(0.0)
    }
}

/// Weather susceptibility for an activity: expected lost working days and the
/// working hours lost per affected day.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherExposure {
    /// Expected number of lost working days across the activity.
    pub expected_lost_days: f64,
    /// Working hours lost per weather-affected day.
    pub hours_per_lost_day: f64,
}

/// An activity within a risk model: a base [`Activity`] plus optional duration
/// and cost uncertainty and weather exposure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiskActivity {
    /// The base activity (carries the identifier, name, and nominal duration).
    pub activity: Activity,
    /// Optional duration uncertainty; when absent the nominal duration is used.
    #[serde(default)]
    pub duration_dist: Option<DurationDistribution>,
    /// Optional cost uncertainty; when absent the activity contributes zero cost.
    #[serde(default)]
    pub cost_dist: Option<CostDistribution>,
    /// Optional weather exposure adding lost-time uncertainty.
    #[serde(default)]
    pub weather: Option<WeatherExposure>,
}

impl RiskActivity {
    /// Wrap a base activity with no uncertainty.
    pub fn new(activity: Activity) -> Self {
        Self {
            activity,
            duration_dist: None,
            cost_dist: None,
            weather: None,
        }
    }

    /// Attach a duration distribution.
    pub fn with_duration_dist(mut self, dist: DurationDistribution) -> Self {
        self.duration_dist = Some(dist);
        self
    }

    /// Attach a cost distribution.
    pub fn with_cost_dist(mut self, dist: CostDistribution) -> Self {
        self.cost_dist = Some(dist);
        self
    }

    /// Attach weather exposure.
    pub fn with_weather(mut self, weather: WeatherExposure) -> Self {
        self.weather = Some(weather);
        self
    }
}

/// A complete risk model: activities plus the dependencies and constraints that
/// define the project network.
#[derive(Clone, Debug, Default)]
pub struct RiskModel {
    activities: Vec<RiskActivity>,
    deps: Vec<Dependency>,
    constraints: Vec<ActivityConstraint>,
}

impl RiskModel {
    /// Create an empty risk model.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a risk activity.
    pub fn add_activity(&mut self, activity: RiskActivity) {
        self.activities.push(activity);
    }

    /// Mutable access to the activity list (e.g. to attach weather exposure).
    pub fn activities_mut(&mut self) -> &mut Vec<RiskActivity> {
        &mut self.activities
    }

    /// Add a precedence dependency (copied into each simulation).
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.deps.push(dep);
    }

    /// Add an activity constraint (copied into each simulation).
    pub fn add_constraint(&mut self, constraint: ActivityConstraint) {
        self.constraints.push(constraint);
    }

    /// Run `iterations` Monte Carlo trials and summarise the outcomes.
    pub fn simulate(&self, iterations: u32, seed: u64) -> Result<SimulationResult, RiskError> {
        if self.activities.is_empty() {
            return Err(RiskError::EmptyModel);
        }
        if iterations == 0 {
            return Err(RiskError::NoIterations);
        }
        let mut rng = Rng::new(seed);
        let mut durations: Vec<f64> = Vec::with_capacity(iterations as usize);
        let mut costs: Vec<f64> = Vec::with_capacity(iterations as usize);

        for _ in 0..iterations {
            let mut sampled_dur: HashMap<ActivityId, f64> = HashMap::new();
            let mut sampled_cost: HashMap<ActivityId, f64> = HashMap::new();
            for ra in &self.activities {
                let id = ra.activity.id;
                let nominal = ra.activity.duration.hours();
                let mut dur = match &ra.duration_dist {
                    Some(d) => d.sample(&mut rng),
                    None => nominal,
                };
                if let Some(w) = ra.weather {
                    let lost_days = rng.poisson(w.expected_lost_days);
                    dur += lost_days * w.hours_per_lost_day;
                }
                sampled_dur.insert(id, dur.max(0.0));
                let cost = match &ra.cost_dist {
                    Some(c) => c.sample(&mut rng),
                    None => 0.0,
                };
                sampled_cost.insert(id, cost);
            }

            let mut net = ScheduleNetwork::new();
            for ra in &self.activities {
                let mut a = ra.activity.clone();
                a.duration = Duration::from_hours(sampled_dur[&a.id]);
                net.add_activity(a)?;
            }
            for d in &self.deps {
                net.add_dependency(*d)?;
            }
            for c in &self.constraints {
                net.add_constraint(*c)?;
            }
            let result = net.schedule()?;
            durations.push(result.project_duration);
            costs.push(sampled_cost.values().sum());
        }

        Ok(SimulationResult::new(durations, costs))
    }
}

/// Summary statistics over a simulated sample.
#[derive(Clone, Debug, PartialEq)]
pub struct Statistics {
    /// Arithmetic mean of the sample.
    pub mean: f64,
    /// Population standard deviation.
    pub std_dev: f64,
    /// 50th percentile (median).
    pub p50: f64,
    /// 80th percentile.
    pub p80: f64,
    /// 90th percentile.
    pub p90: f64,
}

impl Statistics {
    fn from_sample(mut v: Vec<f64>) -> Self {
        let n = v.len().max(1) as f64;
        let mean = v.iter().sum::<f64>() / n;
        let variance = v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let pct = |p: f64| -> f64 {
            let idx = ((p * (v.len() as f64 - 1.0)).round()) as usize;
            v[idx]
        };
        Statistics {
            mean,
            std_dev: variance.sqrt(),
            p50: pct(0.50),
            p80: pct(0.80),
            p90: pct(0.90),
        }
    }
}

/// The aggregated result of a Monte Carlo simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationResult {
    /// Raw project-duration samples (working hours).
    pub durations: Vec<f64>,
    /// Raw total-cost samples (project currency).
    pub costs: Vec<f64>,
    /// Summary statistics for project duration.
    pub schedule: Statistics,
    /// Summary statistics for total cost.
    pub cost: Statistics,
}

impl SimulationResult {
    fn new(durations: Vec<f64>, costs: Vec<f64>) -> Self {
        let schedule = Statistics::from_sample(durations.clone());
        let cost = Statistics::from_sample(costs.clone());
        Self {
            durations,
            costs,
            schedule,
            cost,
        }
    }

    /// Probability (0–1) that the project finishes by `target_hours`.
    pub fn probability_schedule_met(&self, target_hours: f64) -> f64 {
        let met = self
            .durations
            .iter()
            .filter(|&&d| d <= target_hours)
            .count();
        met as f64 / self.durations.len() as f64
    }

    /// Probability (0–1) that total cost stays at or below `target`.
    pub fn probability_cost_met(&self, target: f64) -> f64 {
        let met = self.costs.iter().filter(|&&c| c <= target).count();
        met as f64 / self.costs.len() as f64
    }

    /// Cost contingency (above `baseline`) needed to cover `confidence` of
    /// outcomes (e.g. `0.9` for the P90).
    pub fn cost_contingency(&self, baseline: f64, confidence: f64) -> f64 {
        let target = percentile(&self.costs, confidence);
        (target - baseline).max(0.0)
    }
}

/// Percentile of an unsorted sample at `p` (0–1), nearest-rank.
fn percentile(sample: &[f64], p: f64) -> f64 {
    if sample.is_empty() {
        return 0.0;
    }
    let mut v: Vec<f64> = sample.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p * (v.len() as f64 - 1.0)).round()) as usize;
    v[idx]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_units::Duration;

    fn act(id: u8, hours: f64) -> Activity {
        Activity::new(
            ActivityId::from_uuid(uuid::Uuid::from_u128(id as u128)),
            format!("A{id}"),
            Duration::from_hours(hours),
        )
    }

    fn id(id: u8) -> ActivityId {
        ActivityId::from_uuid(uuid::Uuid::from_u128(id as u128))
    }

    #[test]
    fn deterministic_simulation() {
        let mut m = RiskModel::new();
        m.add_activity(
            RiskActivity::new(act(1, 10.0))
                .with_duration_dist(DurationDistribution::Uniform {
                    min_hours: 8.0,
                    max_hours: 12.0,
                })
                .with_cost_dist(CostDistribution::Triangular {
                    min: 100.0,
                    mode: 120.0,
                    max: 150.0,
                }),
        );
        m.add_activity(RiskActivity::new(act(2, 5.0)).with_duration_dist(
            DurationDistribution::Normal {
                mean_hours: 5.0,
                sd_hours: 1.0,
            },
        ));
        m.add_dependency(Dependency::finish_to_start(id(1), id(2)));

        let r1 = m.simulate(2000, 123).unwrap();
        let r2 = m.simulate(2000, 123).unwrap();
        assert_eq!(r1.durations, r2.durations);
        assert!(
            (r1.schedule.mean - 15.0).abs() < 0.6,
            "mean {}",
            r1.schedule.mean
        );
        assert!(r1.schedule.p90 >= r1.schedule.p50);
        assert!(r1.cost.mean > 0.0);
    }

    #[test]
    fn weather_adds_time() {
        let mut m = RiskModel::new();
        m.add_activity(
            RiskActivity::new(act(1, 10.0)).with_weather(WeatherExposure {
                expected_lost_days: 5.0,
                hours_per_lost_day: 8.0,
            }),
        );
        let r = m.simulate(3000, 5).unwrap();
        assert!(
            r.schedule.mean > 10.0 + 5.0 * 8.0 - 5.0,
            "mean {}",
            r.schedule.mean
        );
    }

    #[test]
    fn probability_and_contingency() {
        let mut m = RiskModel::new();
        m.add_activity(
            RiskActivity::new(act(1, 10.0)).with_cost_dist(CostDistribution::Uniform {
                min: 90.0,
                max: 110.0,
            }),
        );
        let r = m.simulate(5000, 9).unwrap();
        // With a 100 target and 90–110 uniform cost, ~50% should meet it.
        let p = r.probability_cost_met(100.0);
        assert!((p - 0.5).abs() < 0.1, "p {p}");
        assert!(r.cost_contingency(100.0, 0.9) >= 0.0);
    }

    #[test]
    fn guards() {
        let m = RiskModel::new();
        assert_eq!(m.simulate(10, 1), Err(RiskError::EmptyModel));
        let mut m2 = RiskModel::new();
        m2.add_activity(RiskActivity::new(act(1, 1.0)));
        assert_eq!(m2.simulate(0, 1), Err(RiskError::NoIterations));
    }
}
