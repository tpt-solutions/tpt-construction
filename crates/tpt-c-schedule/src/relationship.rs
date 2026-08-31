// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Precedence relationships (dependencies) between activities.

use serde::{Deserialize, Serialize};

use tpt_c_core::ActivityId;

/// The four logical relationship types used in precedence-diagram scheduling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    /// Successor cannot start until the predecessor finishes.
    FinishToStart,
    /// Successor cannot start until the predecessor starts.
    StartToStart,
    /// Successor cannot finish until the predecessor finishes.
    FinishToFinish,
    /// Successor cannot finish until the predecessor starts.
    StartToFinish,
}

/// A precedence dependency from one activity to another.
///
/// `lag_hours` is a delay (positive) or lead (negative) measured in working
/// hours and applied between the two activities per [`RelationshipType`].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    /// The activity that must occur first.
    pub predecessor: ActivityId,
    /// The activity constrained by the predecessor.
    pub successor: ActivityId,
    /// The logical relationship between them.
    pub kind: RelationshipType,
    /// Lag (positive) or lead (negative) in working hours. Defaults to zero.
    #[serde(default)]
    pub lag_hours: f64,
}

impl Dependency {
    /// A Finish-to-Start dependency with zero lag.
    pub fn finish_to_start(predecessor: ActivityId, successor: ActivityId) -> Self {
        Self {
            predecessor,
            successor,
            kind: RelationshipType::FinishToStart,
            lag_hours: 0.0,
        }
    }

    /// Apply a lag (or lead, if negative) to the dependency.
    pub fn with_lag(mut self, lag_hours: f64) -> Self {
        self.lag_hours = lag_hours;
        self
    }
}
