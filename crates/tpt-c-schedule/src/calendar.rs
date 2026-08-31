// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Working-time calendars used to project the CPM timeline onto real dates.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::date::{NaiveDate, Weekday};

/// A recurring work calendar.
///
/// Defines which weekdays are worked, how many hours each working day carries,
/// and an optional set of holiday dates that are never worked. All projections
/// are day-granular: a task that consumes a partial working day is reported as
/// occurring on the day its first working hour falls.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calendar {
    /// Weekdays that count as working days.
    pub working_days: HashSet<Weekday>,
    /// Working hours available on each working day.
    pub hours_per_day: f64,
    /// Explicit non-working dates (holidays, shutdowns).
    #[serde(default)]
    pub holidays: HashSet<NaiveDate>,
}

impl Calendar {
    /// A standard Monday–Friday, 8-hours-per-day calendar (40h work week).
    pub fn standard_40h() -> Self {
        let mut working_days = HashSet::new();
        working_days.insert(Weekday::Monday);
        working_days.insert(Weekday::Tuesday);
        working_days.insert(Weekday::Wednesday);
        working_days.insert(Weekday::Thursday);
        working_days.insert(Weekday::Friday);
        Calendar {
            working_days,
            hours_per_day: 8.0,
            holidays: HashSet::new(),
        }
    }

    /// Whether `date` is a working day under this calendar.
    pub fn is_working(&self, date: NaiveDate) -> bool {
        self.working_days.contains(&date.weekday()) && !self.holidays.contains(&date)
    }

    /// Project `hours` of working time starting at the beginning of `start`,
    /// returning the calendar date on which that working time is exhausted.
    ///
    /// A non-working `start` is skipped forward to the first working day before
    /// any time is consumed.
    pub fn add_working_hours(&self, start: NaiveDate, hours: f64) -> NaiveDate {
        if hours <= 0.0 {
            return start;
        }
        let mut date = start;
        let mut remaining = hours;
        loop {
            if self.is_working(date) {
                if remaining <= self.hours_per_day {
                    return date;
                }
                remaining -= self.hours_per_day;
            }
            date = date.next_day();
        }
    }

    /// Total working hours available between `start` (inclusive) and `end`
    /// (exclusive).
    pub fn working_hours_between(&self, start: NaiveDate, end: NaiveDate) -> f64 {
        let mut total = 0.0;
        let mut d = start;
        while d < end {
            if self.is_working(d) {
                total += self.hours_per_day;
            }
            d = d.next_day();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_projection() {
        let cal = Calendar::standard_40h();
        let mon = NaiveDate::new(2024, 1, 1);
        assert_eq!(cal.add_working_hours(mon, 8.0), NaiveDate::new(2024, 1, 1));
        assert_eq!(cal.add_working_hours(mon, 16.0), NaiveDate::new(2024, 1, 2));
        assert_eq!(cal.add_working_hours(mon, 40.0), NaiveDate::new(2024, 1, 5));
        assert_eq!(cal.add_working_hours(mon, 48.0), NaiveDate::new(2024, 1, 8));
    }

    #[test]
    fn skips_weekend() {
        let cal = Calendar::standard_40h();
        let fri = NaiveDate::new(2024, 1, 5);
        assert_eq!(cal.add_working_hours(fri, 8.0), NaiveDate::new(2024, 1, 5));
        // One more day rolls past Saturday/Sunday to Monday.
        assert_eq!(cal.add_working_hours(fri, 16.0), NaiveDate::new(2024, 1, 8));
    }

    #[test]
    fn working_hours_count() {
        let cal = Calendar::standard_40h();
        let mon = NaiveDate::new(2024, 1, 1);
        let fri = NaiveDate::new(2024, 1, 5);
        assert_eq!(cal.working_hours_between(mon, fri), 32.0);
    }
}
