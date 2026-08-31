// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Baselines and progress actuals for schedule control.

use serde::{Deserialize, Serialize};

use tpt_c_core::ActivityId;

use crate::cpm::CpmResult;

/// A frozen snapshot of a solved schedule, used for variance reporting.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Baseline {
    /// Identifier for the baseline (often the name).
    pub id: String,
    /// Human-readable baseline name (e.g. "Bid Schedule", "Approved v2").
    pub name: String,
    /// RFC 3339 timestamp the baseline was captured.
    pub created_at: String,
    /// The captured CPM result.
    pub result: CpmResult,
}

impl Baseline {
    /// Capture a baseline from a solved result.
    pub fn new(name: impl Into<String>, created_at: impl Into<String>, result: CpmResult) -> Self {
        let name = name.into();
        let id = name.clone();
        Self {
            id,
            name,
            created_at: created_at.into(),
            result,
        }
    }
}

/// Reported progress and cost for a single activity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Actuals {
    /// The activity these actuals describe.
    pub activity: ActivityId,
    /// Actual start time in working hours, if begun.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_start_hour: Option<f64>,
    /// Actual finish time in working hours, if complete.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_finish_hour: Option<f64>,
    /// Percent complete, 0.0–100.0.
    #[serde(default)]
    pub percent_complete: f64,
    /// Actual cost incurred, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual_cost: Option<f64>,
}

impl Actuals {
    /// Create empty actuals for an activity.
    pub fn new(activity: ActivityId) -> Self {
        Self {
            activity,
            actual_start_hour: None,
            actual_finish_hour: None,
            percent_complete: 0.0,
            actual_cost: None,
        }
    }

    /// Record the actual start time.
    pub fn started(mut self, hour: f64) -> Self {
        self.actual_start_hour = Some(hour);
        self
    }

    /// Record the actual finish time.
    pub fn finished(mut self, hour: f64) -> Self {
        self.actual_finish_hour = Some(hour);
        self
    }

    /// Set percent complete (0–100).
    pub fn with_percent(mut self, percent: f64) -> Self {
        self.percent_complete = percent;
        self
    }

    /// Record actual cost.
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.actual_cost = Some(cost);
        self
    }

    /// Schedule variance in hours (actual finish − planned finish).
    /// Negative means the activity finished ahead of plan.
    pub fn finish_variance_hours(&self, planned_finish: f64) -> Option<f64> {
        self.actual_finish_hour.map(|f| f - planned_finish)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::ActivityId;
    use tpt_c_units::Duration;
    use crate::activity::Activity;
    use crate::ScheduleNetwork;

    #[test]
    fn actuals_variance() {
        let id = ActivityId::nil();
        let a = Actuals::new(id).finished(12.0);
        assert_eq!(a.finish_variance_hours(10.0), Some(2.0));
        assert_eq!(Actuals::new(id).finish_variance_hours(10.0), None);
    }

    #[test]
    fn baseline_captures_result() {
        let mut net = ScheduleNetwork::new();
        net.add_activity(Activity::new(ActivityId::nil(), "A", Duration::from_hours(5.0)))
            .unwrap();
        let result = net.schedule().unwrap();
        let base = Baseline::new("v1", "2026-01-01T00:00:00Z", result);
        assert_eq!(base.name, "v1");
        assert!((base.result.project_duration - 5.0).abs() < 1e-9);
    }
}
