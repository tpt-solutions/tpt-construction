// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Toolbox talks.

use serde::{Deserialize, Serialize};

/// A toolbox talk (briefing) delivered on site.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolboxTalk {
    /// Identifier / reference.
    pub id: String,
    /// Topic discussed.
    pub topic: String,
    /// Date of the talk (RFC 3339 date).
    pub date: String,
    /// Attendees (names / ids).
    #[serde(default)]
    pub attendees: Vec<String>,
}

impl ToolboxTalk {
    /// Create a toolbox talk.
    pub fn new(topic: impl Into<String>, date: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            topic: topic.into(),
            date: date.into(),
            attendees: Vec::new(),
        }
    }

    /// Set an explicit reference id.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Record an attendee.
    pub fn add_attendee(&mut self, name: impl Into<String>) {
        self.attendees.push(name.into());
    }

    /// Number of attendees.
    pub fn attendee_count(&self) -> usize {
        self.attendees.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_id_and_attendees() {
        let mut t = ToolboxTalk::new("Crane lift plan", "2026-03-02").with_id("TBT-7");
        t.add_attendee("dave");
        assert_eq!(t.id, "TBT-7");
        assert_eq!(t.attendee_count(), 1);
    }
}
