// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Manpower tracking on a daily log.

use serde::{Deserialize, Serialize};

/// A trade or labour category on site.
pub type Trade = String;

/// A manpower record: a trade, headcount, and hours worked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ManpowerRecord {
    /// Trade or labour category (e.g. "carpenters").
    pub trade: Trade,
    /// Number of workers.
    pub headcount: u32,
    /// Hours worked by the crew (not per-person).
    pub hours: f64,
}

impl ManpowerRecord {
    /// Create a manpower record.
    pub fn new(trade: impl Into<String>, headcount: u32, hours: f64) -> Self {
        Self {
            trade: trade.into(),
            headcount,
            hours,
        }
    }

    /// Total labour hours for this record (headcount × hours).
    pub fn total_hours(&self) -> f64 {
        self.headcount as f64 * self.hours
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_hours_reflects_crew() {
        let m = ManpowerRecord::new("labourers", 5, 8.0);
        assert_eq!(m.total_hours(), 40.0);
    }
}
