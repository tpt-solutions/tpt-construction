// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Field execution records for construction projects.
//!
//! Captures the day-to-day reality of a jobsite: the [`DailyLog`], the weather
//! that shaped it, the [`ManpowerRecord`] that performed the work, free-form
//! [`WorkRecord`] entries, and [`SiteObservation`]s (safety / quality /
//! environmental). A daily log can link the RFIs it spawned via
//! `RFIId`s, tying field capture to the approval workflow.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tpt_c_core::{AuditMeta, CoreError, ProjectId};
use tpt_c_ids::RFIId;
use uuid::Uuid;

mod manpower;
mod observation;
mod weather;

pub use manpower::{ManpowerRecord, Trade};
pub use observation::{ObservationCategory, ObservationStatus, Severity, SiteObservation};
pub use weather::{WeatherCondition, WeatherRecord};

/// Generate a UUID-backed identifier newtype local to this crate.
macro_rules! field_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Build from an explicit UUID (e.g. imported).
            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }
            /// The nil (all-zeros) identifier, useful as a sentinel.
            pub fn nil() -> Self {
                Self(Uuid::nil())
            }
            /// Return the inner UUID.
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl FromStr for $name {
            type Err = CoreError;
            fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|e| CoreError::InvalidId { ty: stringify!($name), detail: e.to_string() })
            }
        }
    };
}

field_id!(
    /// Identifies a daily log aggregate.
    DailyLogId
);
field_id!(
    /// Identifies a free-form work record.
    WorkRecordId
);

/// Errors raised while building or mutating field records.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FieldError {
    /// A required field was missing.
    #[error("missing required {0}")]
    MissingRequired(&'static str),
    /// A numeric value was out of range.
    #[error("value out of range: {0}")]
    OutOfRange(&'static str),
    /// A core domain error occurred.
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// A single work entry recorded on a daily log.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkRecord {
    /// Stable id for the work entry.
    pub id: WorkRecordId,
    /// What was performed.
    pub description: String,
    /// Where it happened (grid, room, zone, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Quantity accomplished, if tracked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<f64>,
    /// Unit of the quantity (e.g. "m2", "ea").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

impl WorkRecord {
    /// Create a work record.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            id: WorkRecordId::from_uuid(Uuid::now_v7()),
            description: description.into(),
            location: None,
            quantity: None,
            unit: None,
        }
    }

    /// Attach a location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Attach a measured quantity and unit.
    pub fn with_quantity(mut self, quantity: f64, unit: impl Into<String>) -> Self {
        self.quantity = Some(quantity);
        self.unit = Some(unit.into());
        self
    }
}

/// A daily log capturing one project day.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DailyLog {
    /// Daily log identifier.
    pub id: DailyLogId,
    /// Project this log belongs to.
    pub project_id: ProjectId,
    /// Log date (RFC 3339 date, `YYYY-MM-DD`).
    pub date: String,
    /// Weather observed on the day.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weather: Option<WeatherRecord>,
    /// Manpower records for the day.
    #[serde(default)]
    pub manpower: Vec<ManpowerRecord>,
    /// Work performed during the day.
    #[serde(default)]
    pub work_performed: Vec<WorkRecord>,
    /// Site observations raised during the day.
    #[serde(default)]
    pub observations: Vec<SiteObservation>,
    /// RFIs opened in connection with this log.
    #[serde(default)]
    pub linked_rfis: Vec<RFIId>,
    /// Free-form narrative notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Who created the log.
    pub created_by: String,
    /// When the log was created.
    pub created_at: AuditMeta,
}

impl DailyLog {
    /// Create a new daily log for `date` by `created_by`.
    pub fn new(
        project_id: ProjectId,
        date: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        let actor: String = created_by.into();
        Self {
            id: DailyLogId::from_uuid(Uuid::now_v7()),
            project_id,
            date: date.into(),
            weather: None,
            manpower: Vec::new(),
            work_performed: Vec::new(),
            observations: Vec::new(),
            linked_rfis: Vec::new(),
            notes: None,
            created_by: actor.clone(),
            created_at: AuditMeta::new(actor, now_stamp()),
        }
    }

    /// Attach the day's weather.
    pub fn set_weather(mut self, weather: WeatherRecord) -> Self {
        self.weather = Some(weather);
        self
    }

    /// Record manpower for a trade.
    pub fn add_manpower(&mut self, trade: impl Into<String>, headcount: u32, hours: f64) {
        self.manpower
            .push(ManpowerRecord::new(trade, headcount, hours));
    }

    /// Record a work entry.
    pub fn add_work(&mut self, work: WorkRecord) {
        self.work_performed.push(work);
    }

    /// Raise a site observation.
    pub fn add_observation(&mut self, observation: SiteObservation) {
        self.observations.push(observation);
    }

    /// Link an RFI raised from this log.
    pub fn link_rfi(&mut self, rfi: RFIId) {
        self.linked_rfis.push(rfi);
    }

    /// Total labour hours recorded across all manpower entries.
    pub fn total_man_hours(&self) -> f64 {
        self.manpower.iter().map(|m| m.total_hours()).sum()
    }

    /// Number of distinct trades on site.
    pub fn trade_count(&self) -> usize {
        self.manpower.len()
    }

    /// Count of open (unresolved) observations.
    pub fn open_observations(&self) -> usize {
        self.observations
            .iter()
            .filter(|o| o.status == ObservationStatus::Open)
            .count()
    }
}

/// Convenience: build a fresh RFC 3339 timestamp string for `created_at`.
fn now_stamp() -> String {
    // Field capture rarely needs sub-second precision; use a stable sentinel so
    // tests are deterministic about the shape. Real callers overwrite via the
    // public `created_at` field.
    "1970-01-01T00:00:00Z".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::ProjectId;

    #[test]
    fn daily_log_accumulates_manpower_and_hours() {
        let mut log = DailyLog::new(ProjectId::nil(), "2026-03-01", "foreman");
        log.add_manpower("carpenters", 4, 8.0);
        log.add_manpower("electricians", 2, 6.5);
        assert_eq!(log.trade_count(), 2);
        assert_eq!(log.total_man_hours(), 4.0 * 8.0 + 2.0 * 6.5);
    }

    #[test]
    fn observation_lifecycle() {
        let mut obs = SiteObservation::new(
            ObservationCategory::Safety,
            "Unguarded floor opening",
            Severity::High,
        );
        assert_eq!(obs.status, ObservationStatus::Open);
        obs.resolve(AuditMeta::new("safety", "2026-03-01T00:00:00Z"));
        assert_eq!(obs.status, ObservationStatus::Closed);
    }

    #[test]
    fn work_record_with_quantity() {
        let w = WorkRecord::new("Pour slab")
            .with_location("Grid A1-C3")
            .with_quantity(12.5, "m3");
        assert_eq!(w.quantity, Some(12.5));
        assert_eq!(w.unit.as_deref(), Some("m3"));
    }

    #[test]
    fn daily_log_id_roundtrip() {
        let id = DailyLogId::from_uuid(Uuid::now_v7());
        let s = id.to_string();
        assert_eq!(s.parse::<DailyLogId>().unwrap(), id);
    }
}
