// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construction scheduling for TPT Construction.
//!
//! This crate models a project schedule as a network of [`Activity`] nodes
//! linked by [`Dependency`] relationships (Finish-to-Start, Start-to-Start,
//! Finish-to-Finish, Start-to-Finish) with optional lag or lead. A
//! [`Calendar`] projects the engine's internal **working-hour** timeline onto
//! real calendar dates, and the [`ScheduleNetwork::schedule`] method runs the
//! full Critical Path Method (CPM) forward/backward pass to compute early and
//! late dates, total and free float, and the critical path.
//!
//! Baselines ([`Baseline`]) and progress [`Actuals`] round out the schedule
//! lifecycle: capture a plan snapshot, then compare actual start/finish and
//! percent-complete against it.
//!
//! The crate is intentionally free of optional external scheduler substrates so
//! it builds fully self-contained (see spec §4).

mod activity;
mod aggregate;
mod baseline;
mod calendar;
mod constraint;
mod cpm;
mod date;
mod relationship;

pub use activity::{Activity, ActivityStatus};
pub use aggregate::Schedule;
pub use baseline::{Actuals, Baseline};
pub use calendar::{Calendar, Weekday};
pub use constraint::{ActivityConstraint, ConstraintType};
pub use cpm::{ActivitySchedule, CpmResult, ScheduleNetwork};
pub use date::NaiveDate;
pub use relationship::{Dependency, RelationshipType};
pub use tpt_c_core::{ActivityId, ScheduleId};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors raised while building or solving a schedule network.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScheduleError {
    /// An activity was added whose identifier is already present.
    #[error("activity {0} already exists")]
    DuplicateActivity(ActivityId),

    /// A referenced activity identifier is not in the network.
    #[error("unknown activity {0}")]
    UnknownActivity(ActivityId),

    /// A dependency referenced an activity that is not in the network.
    #[error("dependency references an unknown activity")]
    DanglingDependency,

    /// The dependency graph contains a cycle and cannot be topologically ordered.
    #[error("cycle detected in schedule network")]
    CycleDetected,

    /// A date string could not be parsed into a [`NaiveDate`].
    #[error("invalid date string: {0}")]
    InvalidDate(String),

    /// The network has no activities to schedule.
    #[error("empty schedule: no activities")]
    EmptySchedule,
}

/// Convenience `Result` alias for schedule operations.
pub type Result<T> = std::result::Result<T, ScheduleError>;
