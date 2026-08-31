// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The [`Schedule`] aggregate: a network plus calendar and project start.

use serde::{Deserialize, Serialize};

use tpt_c_core::{ActivityId, ScheduleId};

use crate::calendar::Calendar;
use crate::cpm::{CpmResult, ScheduleNetwork};
use crate::date::NaiveDate;
use crate::Activity;
use crate::ActivityConstraint;
use crate::Dependency;
use crate::ScheduleError;

/// A complete, named project schedule.
///
/// Wraps a [`ScheduleNetwork`] together with the [`Calendar`] and project start
/// date needed to project the CPM working-hour timeline onto real dates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    /// Identifier for this schedule aggregate.
    pub id: ScheduleId,
    /// Human-readable schedule name.
    pub name: String,
    /// The working-time calendar.
    pub calendar: Calendar,
    /// The project start date (working hour 0).
    pub project_start: NaiveDate,
    network: ScheduleNetwork,
}

impl Schedule {
    /// Create a schedule with a calendar and start date.
    pub fn new(name: impl Into<String>, calendar: Calendar, project_start: NaiveDate) -> Self {
        Self {
            id: ScheduleId::nil(),
            name: name.into(),
            calendar,
            project_start,
            network: ScheduleNetwork::new(),
        }
    }

    /// Assign a generated schedule identifier.
    pub fn with_id(mut self, id: ScheduleId) -> Self {
        self.id = id;
        self
    }

    /// Add an activity to the schedule.
    pub fn add_activity(&mut self, activity: Activity) -> Result<(), ScheduleError> {
        self.network.add_activity(activity)
    }

    /// Add a precedence dependency.
    pub fn add_dependency(&mut self, dep: Dependency) -> Result<(), ScheduleError> {
        self.network.add_dependency(dep)
    }

    /// Add an activity constraint.
    pub fn add_constraint(&mut self, constraint: ActivityConstraint) -> Result<(), ScheduleError> {
        self.network.add_constraint(constraint)
    }

    /// Solve the schedule with the CPM engine.
    pub fn compute(&self) -> Result<CpmResult, ScheduleError> {
        self.network.schedule()
    }

    /// All activities in the schedule, keyed by identifier.
    pub fn activities(&self) -> &std::collections::HashMap<ActivityId, Activity> {
        self.network.activities()
    }

    /// Calendar start and finish dates for an activity's early dates.
    pub fn activity_dates(
        &self,
        result: &CpmResult,
        id: ActivityId,
    ) -> Result<(NaiveDate, NaiveDate), ScheduleError> {
        let s = result
            .activities
            .get(&id)
            .ok_or(ScheduleError::UnknownActivity(id))?;
        let start = self
            .calendar
            .add_working_hours(self.project_start, s.early_start);
        let finish = self
            .calendar
            .add_working_hours(self.project_start, s.early_finish);
        Ok((start, finish))
    }

    /// The calendar date on which the project is forecast to finish.
    pub fn project_finish_date(&self, result: &CpmResult) -> NaiveDate {
        self.calendar
            .add_working_hours(self.project_start, result.project_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_units::Duration;

    #[test]
    fn project_dates_projected() {
        let cal = Calendar::standard_40h();
        let start = NaiveDate::new(2024, 1, 1);
        let mut sched = Schedule::new("Demo", cal, start);
        sched
            .add_activity(Activity::new(
                ActivityId::nil(),
                "A",
                Duration::from_hours(16.0),
            ))
            .unwrap();
        let r = sched.compute().unwrap();
        let (s, f) = sched.activity_dates(&r, ActivityId::nil()).unwrap();
        assert_eq!(s, NaiveDate::new(2024, 1, 1));
        assert_eq!(f, NaiveDate::new(2024, 1, 2));
        assert_eq!(sched.project_finish_date(&r), NaiveDate::new(2024, 1, 2));
    }
}
