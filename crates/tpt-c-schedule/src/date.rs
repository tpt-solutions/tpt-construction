// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Calendar date arithmetic without external dependencies.
//!
//! [`NaiveDate`] stores a year/month/day and converts to/from a day count using
//! Howard Hinnant's `days_from_civil` / `civil_from_days` algorithms, which are
//! valid for the full range of civil dates. [`Weekday`] and [`NaiveDate::weekday`]
//! derive the day of the week directly from that count.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::ScheduleError;

/// A day-level calendar date with no time zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NaiveDate {
    /// Calendar year (e.g. 2026).
    pub year: i32,
    /// Calendar month, 1–12.
    pub month: u32,
    /// Calendar day of month, 1–31.
    pub day: u32,
}

/// Day of the week.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weekday {
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl NaiveDate {
    /// Build a date from its parts. No validation beyond basic range is performed.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Days since 1970-01-01 (Hinnant's civil algorithm).
    pub fn to_ordinal(self) -> i64 {
        let y = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = if self.month > 2 {
            self.month - 3
        } else {
            self.month + 9
        };
        let doy = (153 * mp as i64 + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// Build a date from a day count returned by [`NaiveDate::to_ordinal`].
    pub fn from_ordinal(z: i64) -> Self {
        let z = z + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };
        NaiveDate {
            year: y as i32,
            month: m as u32,
            day: d as u32,
        }
    }

    /// The day of the week for this date.
    pub fn weekday(self) -> Weekday {
        let w = ((self.to_ordinal() % 7) + 4).rem_euclid(7);
        match w {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            _ => Weekday::Saturday,
        }
    }

    /// The next calendar day.
    pub fn next_day(self) -> Self {
        Self::from_ordinal(self.to_ordinal() + 1)
    }

    /// This date shifted by `days` (may be negative).
    pub fn add_days(self, days: i64) -> Self {
        Self::from_ordinal(self.to_ordinal() + days)
    }
}

impl fmt::Display for NaiveDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for NaiveDate {
    type Err = ScheduleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(ScheduleError::InvalidDate(s.to_string()));
        }
        let parse = |p: &str| p.parse::<i32>().map_err(|_| ScheduleError::InvalidDate(s.to_string()));
        let year = parse(parts[0])?;
        let month = parse(parts[1])? as u32;
        let day = parse(parts[2])? as u32;
        Ok(NaiveDate::new(year, month, day))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_weekdays() {
        assert_eq!(NaiveDate::new(2024, 1, 1).weekday(), Weekday::Monday);
        assert_eq!(NaiveDate::new(1970, 1, 1).weekday(), Weekday::Thursday);
        assert_eq!(NaiveDate::new(2000, 1, 1).weekday(), Weekday::Saturday);
        assert_eq!(NaiveDate::new(2026, 8, 31).weekday(), Weekday::Monday);
    }

    #[test]
    fn ordinal_roundtrip() {
        for d in [
            NaiveDate::new(2024, 1, 1),
            NaiveDate::new(2000, 2, 29),
            NaiveDate::new(1900, 3, 1),
            NaiveDate::new(2026, 12, 31),
        ] {
            assert_eq!(NaiveDate::from_ordinal(d.to_ordinal()), d);
        }
    }

    #[test]
    fn next_and_add() {
        let d = NaiveDate::new(2024, 1, 31);
        assert_eq!(d.next_day(), NaiveDate::new(2024, 2, 1));
        assert_eq!(d.add_days(1), NaiveDate::new(2024, 2, 1));
        assert_eq!(NaiveDate::new(2024, 3, 1).add_days(-1), NaiveDate::new(2024, 2, 29));
    }

    #[test]
    fn parse_display() {
        let d: NaiveDate = "2026-08-31".parse().unwrap();
        assert_eq!(d, NaiveDate::new(2026, 8, 31));
        assert_eq!(d.to_string(), "2026-08-31");
        assert!("not-a-date".parse::<NaiveDate>().is_err());
    }
}
