// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Activities: the schedulable units of work.

use serde::{Deserialize, Serialize};

use tpt_c_core::ActivityId;
use tpt_c_units::Duration;

/// Execution state of an activity at a point in time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    /// Work has not begun.
    NotStarted,
    /// Work is in progress.
    InProgress,
    /// Work is complete.
    Completed,
    /// Work is suspended.
    OnHold,
}

/// A single schedulable activity (task) within a project schedule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    /// Stable identifier for the activity.
    pub id: ActivityId,
    /// Human-readable name.
    pub name: String,
    /// Planned duration (in working hours/days via [`tpt_c_units::Duration`]).
    pub duration: Duration,
    /// Current execution status.
    #[serde(default)]
    pub status: ActivityStatus,
}

impl Activity {
    /// Create an activity with a duration and default `NotStarted` status.
    pub fn new(id: ActivityId, name: impl Into<String>, duration: Duration) -> Self {
        Self {
            id,
            name: name.into(),
            duration,
            status: ActivityStatus::NotStarted,
        }
    }

    /// Set the activity's status.
    pub fn with_status(mut self, status: ActivityStatus) -> Self {
        self.status = status;
        self
    }
}
