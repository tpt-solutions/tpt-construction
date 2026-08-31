// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Documents, drawings, specifications, and revision control.

use serde::{Deserialize, Serialize};
use tpt_c_ids::DocumentId;
use uuid::Uuid;

/// The kind of document being controlled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    /// Drawing (plan, detail, elevation).
    Drawing,
    /// Specification section.
    Specification,
    /// Contractual document.
    ContractDocument,
    /// Report (field, progress, commissioning).
    Report,
    /// Anything else.
    Other,
}

/// Lifecycle status of a document.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    /// Draft / not yet issued.
    Draft,
    /// Current issued for construction.
    Current,
    /// Issued but not the latest.
    Issued,
    /// Superseded by a later revision.
    Superseded,
}

/// A single revision of a document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    /// Sequential revision number (1-based).
    pub number: i32,
    /// Who created the revision.
    pub created_by: String,
    /// Description of the change.
    pub note: String,
    /// Optional reference to the file / external system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_ref: Option<String>,
}

/// A controlled document with a revision history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    /// Document identifier.
    pub id: DocumentId,
    /// Title.
    pub title: String,
    /// Document kind.
    pub doc_type: DocumentType,
    /// Current status.
    pub status: DocumentStatus,
    /// Revision history, oldest first.
    #[serde(default)]
    pub revisions: Vec<Revision>,
}

impl Document {
    /// Create a new document in `Draft`.
    pub fn new(id: DocumentId, title: impl Into<String>, doc_type: DocumentType) -> Self {
        Self {
            id,
            title: title.into(),
            doc_type,
            status: DocumentStatus::Draft,
            revisions: Vec::new(),
        }
    }

    /// Add a revision, numbering it sequentially.
    pub fn add_revision(&mut self, note: impl Into<String>) -> i32 {
        let number = (self.revisions.len() as i32) + 1;
        self.revisions.push(Revision {
            number,
            created_by: "author".to_string(),
            note: note.into(),
            file_ref: None,
        });
        if self.status == DocumentStatus::Draft {
            self.status = DocumentStatus::Current;
        }
        number
    }

    /// The latest revision number, or 0 if none.
    pub fn current_revision_no(&self) -> i32 {
        self.revisions.last().map(|r| r.number).unwrap_or(0)
    }

    /// Mark this document superseded.
    pub fn supersede(&mut self) {
        self.status = DocumentStatus::Superseded;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Document {
        Document::new(DocumentId::from_uuid(Uuid::now_v7()), "Drawing A-101", DocumentType::Drawing)
    }

    #[test]
    fn adding_revision_updates_status_and_number() {
        let mut d = doc();
        assert_eq!(d.current_revision_no(), 0);
        let n = d.add_revision("first issue");
        assert_eq!(n, 1);
        assert_eq!(d.status, DocumentStatus::Current);
        d.add_revision("revised");
        assert_eq!(d.current_revision_no(), 2);
    }
}
