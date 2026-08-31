// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Compliance checklists.

use serde::{Deserialize, Serialize};

/// Status of a checklist item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChecklistStatus {
    /// Not yet done.
    Pending,
    /// Satisfied.
    Done,
    /// Not applicable.
    NotApplicable,
}

/// A single item in a compliance checklist.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChecklistItem {
    /// Item description.
    pub description: String,
    /// Status.
    pub status: ChecklistStatus,
}

impl ChecklistItem {
    /// Create a pending item.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            status: ChecklistStatus::Pending,
        }
    }

    /// Mark done.
    pub fn mark_done(&mut self) {
        self.status = ChecklistStatus::Done;
    }

    /// Mark not applicable.
    pub fn mark_na(&mut self) {
        self.status = ChecklistStatus::NotApplicable;
    }
}

/// A compliance checklist used to verify controls.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComplianceChecklist {
    /// Title / reference.
    pub title: String,
    /// Items.
    #[serde(default)]
    pub items: Vec<ChecklistItem>,
}

impl ComplianceChecklist {
    /// Create an empty checklist.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    /// Add an item, returning its index.
    pub fn add_item(&mut self, description: impl Into<String>) -> usize {
        self.items.push(ChecklistItem::new(description));
        self.items.len() - 1
    }

    /// Number of items still pending (excluding N/A).
    pub fn open_items(&self) -> usize {
        self.items
            .iter()
            .filter(|i| i.status == ChecklistStatus::Pending)
            .count()
    }

    /// Whether every applicable item is done.
    pub fn is_complete(&self) -> bool {
        self.open_items() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn na_excluded_from_open() {
        let mut c = ComplianceChecklist::new("Permit to dig");
        let i = c.add_item("Locate services");
        c.add_item("Barricade");
        c.items[i].mark_na();
        assert_eq!(c.open_items(), 1);
        assert!(!c.is_complete());
    }
}
