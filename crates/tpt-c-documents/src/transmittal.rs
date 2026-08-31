// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Transmittals: the formal issue of documents to recipients.

use serde::{Deserialize, Serialize};
use tpt_c_ids::DocumentId;
use tpt_c_workflow::TransmittalWorkflow;

/// A transmittal issuing one or more documents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentTransmittal {
    /// External transmittal number (e.g. "TX-0142").
    pub id: String,
    /// Documents included in the transmittal.
    #[serde(default)]
    pub document_ids: Vec<DocumentId>,
    /// The approval workflow driving the transmittal's state.
    #[serde(flatten)]
    pub workflow: TransmittalWorkflow,
}

impl DocumentTransmittal {
    /// Create a transmittal for the given documents.
    pub fn new(id: impl Into<String>, document_ids: Vec<DocumentId>) -> Self {
        let id: String = id.into();
        Self {
            id: id.clone(),
            document_ids,
            workflow: TransmittalWorkflow::new(id),
        }
    }

    /// Number of documents transmitted.
    pub fn document_count(&self) -> usize {
        self.document_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transmittal_counts_documents() {
        let tx = DocumentTransmittal::new(
            "TX-9",
            vec![DocumentId::nil(), DocumentId::nil()],
        );
        assert_eq!(tx.document_count(), 2);
        assert_eq!(tx.id, "TX-9");
    }
}
