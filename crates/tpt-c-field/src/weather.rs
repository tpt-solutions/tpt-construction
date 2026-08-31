// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Weather records attached to a daily log.

use serde::{Deserialize, Serialize};

/// Observed weather condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeatherCondition {
    /// Clear / sunny.
    Clear,
    /// Partly / mostly cloudy.
    Cloudy,
    /// Rain.
    Rain,
    /// Snow.
    Snow,
    /// High wind.
    Wind,
    /// Storm.
    Storm,
}

/// A weather observation for a day on site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherRecord {
    /// Dominant condition.
    pub condition: WeatherCondition,
    /// Daily high temperature, degrees Celsius.
    pub temp_high_c: f64,
    /// Daily low temperature, degrees Celsius.
    pub temp_low_c: f64,
    /// Precipitation, millimetres.
    #[serde(default)]
    pub precipitation_mm: f64,
    /// Wind speed, metres per second.
    #[serde(default)]
    pub wind_speed_mps: f64,
}

impl WeatherRecord {
    /// Create a weather record.
    pub fn new(condition: WeatherCondition, temp_high_c: f64, temp_low_c: f64) -> Self {
        Self {
            condition,
            temp_high_c,
            temp_low_c,
            precipitation_mm: 0.0,
            wind_speed_mps: 0.0,
        }
    }

    /// Whether the day was a wash-out (storm / heavy precip).
    pub fn is_inclement(&self) -> bool {
        matches!(self.condition, WeatherCondition::Storm | WeatherCondition::Snow)
            || self.precipitation_mm > 5.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inclement_detection() {
        let storm = WeatherRecord::new(WeatherCondition::Storm, 10.0, 4.0);
        assert!(storm.is_inclement());
        let mild = WeatherRecord::new(WeatherCondition::Cloudy, 18.0, 9.0);
        assert!(!mild.is_inclement());
    }
}
