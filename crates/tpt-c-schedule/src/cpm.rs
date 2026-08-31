// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The Critical Path Method (CPM) forward/backward pass.

use std::collections::{HashMap, VecDeque};

use tpt_c_core::ActivityId;

use crate::activity::Activity;
use crate::constraint::{ActivityConstraint, ConstraintType};
use crate::relationship::{Dependency, RelationshipType};
use crate::ScheduleError;

/// Activities within `FLOAT_EPSILON` working hours of zero float are critical.
const FLOAT_EPSILON: f64 = 1e-9;

/// The solved schedule for a single activity.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivitySchedule {
    /// The activity identifier.
    pub id: ActivityId,
    /// Early start (working hours from project start).
    pub early_start: f64,
    /// Early finish (working hours from project start).
    pub early_finish: f64,
    /// Late start (working hours from project start).
    pub late_start: f64,
    /// Late finish (working hours from project start).
    pub late_finish: f64,
    /// Total float: how long the activity may slip without delaying the project.
    pub total_float: f64,
    /// Free float: how long the activity may slip without delaying any successor.
    pub free_float: f64,
    /// Whether the activity lies on a critical path (total float ~ 0).
    pub critical: bool,
}

/// The full result of a CPM solve.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CpmResult {
    /// Per-activity early/late dates, float, and criticality.
    pub activities: HashMap<ActivityId, ActivitySchedule>,
    /// Project duration in working hours (maximum early finish).
    pub project_duration: f64,
    /// Activities on the critical path, in topological order.
    pub critical_path: Vec<ActivityId>,
}

/// A schedulable network of activities, dependencies, and constraints.
#[derive(Clone, Debug, Default)]
pub struct ScheduleNetwork {
    activities: HashMap<ActivityId, Activity>,
    deps: Vec<Dependency>,
    constraints: Vec<ActivityConstraint>,
}

impl ScheduleNetwork {
    /// Create an empty network.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an activity, rejecting duplicate identifiers.
    pub fn add_activity(&mut self, activity: Activity) -> Result<(), ScheduleError> {
        if self.activities.contains_key(&activity.id) {
            return Err(ScheduleError::DuplicateActivity(activity.id));
        }
        self.activities.insert(activity.id, activity);
        Ok(())
    }

    /// Add a precedence dependency, rejecting references to unknown activities.
    pub fn add_dependency(&mut self, dep: Dependency) -> Result<(), ScheduleError> {
        if !self.activities.contains_key(&dep.predecessor)
            || !self.activities.contains_key(&dep.successor)
        {
            return Err(ScheduleError::DanglingDependency);
        }
        if dep.predecessor == dep.successor {
            return Err(ScheduleError::DanglingDependency);
        }
        self.deps.push(dep);
        Ok(())
    }

    /// Add an activity constraint, rejecting references to unknown activities.
    pub fn add_constraint(&mut self, constraint: ActivityConstraint) -> Result<(), ScheduleError> {
        if !self.activities.contains_key(&constraint.activity) {
            return Err(ScheduleError::UnknownActivity(constraint.activity));
        }
        self.constraints.push(constraint);
        Ok(())
    }

    /// All activities in the network.
    pub fn activities(&self) -> &HashMap<ActivityId, Activity> {
        &self.activities
    }

    /// Run the forward and backward CPM passes.
    pub fn schedule(&self) -> Result<CpmResult, ScheduleError> {
        if self.activities.is_empty() {
            return Err(ScheduleError::EmptySchedule);
        }
        let order = self.topological_order()?;
        let dur = |id: &ActivityId| self.activities[id].duration.hours();

        let mut early_start: HashMap<ActivityId, f64> = HashMap::new();
        let mut early_finish: HashMap<ActivityId, f64> = HashMap::new();

        // ---- Forward pass -------------------------------------------------
        for &id in &order {
            let mut start = 0.0;
            for d in self.deps.iter().filter(|d| d.successor == id) {
                let bound = match d.kind {
                    RelationshipType::FinishToStart => early_finish[&d.predecessor] + d.lag_hours,
                    RelationshipType::StartToStart => early_start[&d.predecessor] + d.lag_hours,
                    RelationshipType::FinishToFinish => {
                        early_finish[&d.predecessor] + d.lag_hours - dur(&d.predecessor)
                    }
                    RelationshipType::StartToFinish => {
                        early_start[&d.predecessor] + d.lag_hours - dur(&d.predecessor)
                    }
                };
                if bound > start {
                    start = bound;
                }
            }
            for c in self.constraints.iter().filter(|c| c.activity == id) {
                let min_start = match c.kind {
                    ConstraintType::StartNoEarlierThan | ConstraintType::MustStartOn => c.hours_from_start,
                    ConstraintType::FinishNoEarlierThan | ConstraintType::MustFinishOn => {
                        c.hours_from_start - dur(&id)
                    }
                    _ => f64::NEG_INFINITY,
                };
                if min_start > start {
                    start = min_start;
                }
            }
            early_start.insert(id, start);
            early_finish.insert(id, start + dur(&id));
        }

        let project_duration = early_finish.values().copied().fold(0.0_f64, f64::max);

        // ---- Backward pass ------------------------------------------------
        let mut late_start: HashMap<ActivityId, f64> = HashMap::new();
        let mut late_finish: HashMap<ActivityId, f64> = HashMap::new();

        for &id in order.iter().rev() {
            let mut finish = project_duration;
            for c in self.constraints.iter().filter(|c| c.activity == id) {
                let max_finish = match c.kind {
                    ConstraintType::FinishNoLaterThan | ConstraintType::MustFinishOn => c.hours_from_start,
                    ConstraintType::StartNoLaterThan | ConstraintType::MustStartOn => {
                        c.hours_from_start + dur(&id)
                    }
                    _ => f64::INFINITY,
                };
                if max_finish < finish {
                    finish = max_finish;
                }
            }
            for d in self.deps.iter().filter(|d| d.predecessor == id) {
                let s_ls = late_start[&d.successor];
                let s_lf = s_ls + dur(&d.successor);
                let bound = match d.kind {
                    RelationshipType::FinishToStart => s_ls - d.lag_hours,
                    RelationshipType::StartToStart => s_lf - d.lag_hours,
                    RelationshipType::FinishToFinish => s_ls - d.lag_hours + dur(&id),
                    RelationshipType::StartToFinish => s_lf - d.lag_hours + dur(&id),
                };
                if bound < finish {
                    finish = bound;
                }
            }
            late_finish.insert(id, finish);
            late_start.insert(id, finish - dur(&id));
        }

        // ---- Assemble -----------------------------------------------------
        let mut activities: HashMap<ActivityId, ActivitySchedule> = HashMap::new();
        for &id in &order {
            let es = early_start[&id];
            let ef = early_finish[&id];
            let ls = late_start[&id];
            let lf = late_finish[&id];
            let total_float = ls - es;
            let free_float = self
                .free_float(id, &early_start, &early_finish, dur)
                .unwrap_or(total_float);
            let critical = total_float <= FLOAT_EPSILON;
            activities.insert(
                id,
                ActivitySchedule {
                    id,
                    early_start: es,
                    early_finish: ef,
                    late_start: ls,
                    late_finish: lf,
                    total_float,
                    free_float,
                    critical,
                },
            );
        }

        let critical_path = order
            .iter()
            .filter(|&&id| activities[&id].critical)
            .copied()
            .collect();

        Ok(CpmResult {
            activities,
            project_duration,
            critical_path,
        })
    }

    /// Free float for `id`, or `None` when it has no successors (caller falls
    /// back to total float).
    fn free_float(
        &self,
        id: ActivityId,
        es: &HashMap<ActivityId, f64>,
        ef: &HashMap<ActivityId, f64>,
        dur: impl Fn(&ActivityId) -> f64,
    ) -> Option<f64> {
        let mut free = f64::INFINITY;
        let mut has_successor = false;
        for d in self.deps.iter().filter(|d| d.predecessor == id) {
            has_successor = true;
            let s_es = es[&d.successor];
            let s_ef = s_es + dur(&d.successor);
            let slack = match d.kind {
                RelationshipType::FinishToStart => s_es - d.lag_hours - ef[&id],
                RelationshipType::StartToStart => s_es - d.lag_hours - es[&id],
                RelationshipType::FinishToFinish => s_ef - d.lag_hours - ef[&id],
                RelationshipType::StartToFinish => s_es - d.lag_hours - ef[&id],
            };
            if slack < free {
                free = slack;
            }
        }
        if !has_successor {
            None
        } else if free.is_finite() {
            Some(free)
        } else {
            Some(0.0)
        }
    }

    /// Topologically order activities, detecting cycles.
    fn topological_order(&self) -> Result<Vec<ActivityId>, ScheduleError> {
        let mut indegree: HashMap<ActivityId, usize> = self
            .activities
            .keys()
            .map(|k| (*k, 0))
            .collect();
        for d in &self.deps {
            *indegree.get_mut(&d.successor).unwrap() += 1;
        }
        let mut queue: VecDeque<ActivityId> = indegree
            .iter()
            .filter(|(_, &v)| v == 0)
            .map(|(&k, _)| k)
            .collect();
        let mut order = Vec::new();
        while let Some(n) = queue.pop_front() {
            order.push(n);
            for d in self.deps.iter().filter(|d| d.predecessor == n) {
                let s = indegree.get_mut(&d.successor).unwrap();
                *s -= 1;
                if *s == 0 {
                    queue.push_back(d.successor);
                }
            }
        }
        if order.len() != self.activities.len() {
            return Err(ScheduleError::CycleDetected);
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tpt_c_core::ActivityId;
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
    fn classic_network() {
        let mut net = ScheduleNetwork::new();
        for a in [act(1, 2.0), act(2, 3.0), act(3, 2.0), act(4, 4.0), act(5, 1.0), act(6, 2.0)] {
            net.add_activity(a).unwrap();
        }
        net.add_dependency(Dependency::finish_to_start(id(1), id(2))).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(1), id(3))).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(2), id(4))).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(3), id(5))).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(4), id(6))).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(5), id(6))).unwrap();

        let r = net.schedule().unwrap();
        assert!((r.project_duration - 11.0).abs() < 1e-9);
        assert_eq!(r.activities[&id(1)].early_finish, 2.0);
        assert_eq!(r.activities[&id(6)].early_start, 9.0);

        let critical: HashSet<ActivityId> = r.critical_path.iter().copied().collect();
        assert_eq!(
            critical,
            [id(1), id(2), id(4), id(6)].into_iter().collect()
        );

        // Float for the off-critical branch.
        assert!((r.activities[&id(3)].total_float - 4.0).abs() < 1e-9);
        assert!((r.activities[&id(5)].total_float - 4.0).abs() < 1e-9);
        assert_eq!(r.activities[&id(1)].free_float, 0.0);
    }

    #[test]
    fn lag_shifts_successor() {
        let mut net = ScheduleNetwork::new();
        net.add_activity(act(1, 2.0)).unwrap();
        net.add_activity(act(2, 3.0)).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(1), id(2)).with_lag(1.0))
            .unwrap();
        let r = net.schedule().unwrap();
        assert_eq!(r.activities[&id(2)].early_start, 3.0);
    }

    #[test]
    fn cycle_is_detected() {
        let mut net = ScheduleNetwork::new();
        net.add_activity(act(1, 1.0)).unwrap();
        net.add_activity(act(2, 1.0)).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(1), id(2)))
            .unwrap();
        net.add_dependency(Dependency::finish_to_start(id(2), id(1)))
            .unwrap();
        assert_eq!(net.schedule(), Err(ScheduleError::CycleDetected));
    }

    #[test]
    fn duplicate_and_dangling() {
        let mut net = ScheduleNetwork::new();
        net.add_activity(act(1, 1.0)).unwrap();
        assert_eq!(
            net.add_activity(act(1, 2.0)),
            Err(ScheduleError::DuplicateActivity(id(1)))
        );
        assert_eq!(
            net.add_dependency(Dependency::finish_to_start(id(1), id(9))),
            Err(ScheduleError::DanglingDependency)
        );
    }

    #[test]
    fn constraint_pins_finish() {
        let mut net = ScheduleNetwork::new();
        net.add_activity(act(1, 2.0)).unwrap();
        net.add_activity(act(2, 3.0)).unwrap();
        net.add_dependency(Dependency::finish_to_start(id(1), id(2)))
            .unwrap();
        // Force activity 2 to finish no later than hour 6, compressing its float.
        net.add_constraint(ActivityConstraint::new(
            id(2),
            ConstraintType::FinishNoLaterThan,
            6.0,
        ))
        .unwrap();
        let r = net.schedule().unwrap();
        assert!((r.activities[&id(2)].late_finish - 6.0).abs() < 1e-9);
    }
}
