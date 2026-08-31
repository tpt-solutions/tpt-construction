// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Scheduling constraints applied to individual activities.

use serde::{Deserialize, Serialize};

use tpt_c_core::ActivityId;

/// The kind of constraint placed on an activity's timing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    /// Activity must not start before the given time.
    StartNoEarlierThan,
    /// Activity must not start after the given time.
    StartNoLaterThan,
    /// Activity must not finish before the given time.
    FinishNoEarlierThan,
    /// Activity must not finish after the given time.
    FinishNoLaterThan,
    /// Activity must start exactly at the given time.
    MustStartOn,
    /// Activity must finish exactly at the given time.
    MustFinishOn,
    /// Schedule as early as dependencies permit (default behaviour).
    AsSoonAsPossible,
    /// Schedule as late as dependencies permit.
    AsLateAsPossible,
}

/// A constraint pinning an activity's start or finish to a working-hour offset.
///
/// `hours_from_start` is measured from the project start in working hours, so a
/// constraint expressed as a calendar date should be converted through the
/// project [`crate::Calendar`] before being stored here.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivityConstraint {
    /// The constrained activity.
    pub activity: ActivityId,
    /// The constraint semantics.
    pub kind: ConstraintType,
    /// The constrained time as a working-hour offset from project start.
    pub hours_from_start: f64,
}

impl ActivityConstraint {
    /// Build a constraint of the given kind at a working-hour offset.
    pub fn new(activity: ActivityId, kind: ConstraintType, hours_from_start: f64) -> Self {
        Self {
            activity,
            kind,
            hours_from_start,
        }
    }
}
