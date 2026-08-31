// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Formal notices issued under a contract.

use serde::{Deserialize, Serialize};
use tpt_c_core::AuditMeta;
use tpt_c_ids::NoticeId;
use tpt_c_workflow::NoticeWorkflow;

/// A formal notice (notice of delay, non-conformance, etc.).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    /// Notice identifier.
    pub id: NoticeId,
    /// Subject / title.
    pub subject: String,
    /// Approval workflow driving the notice's state.
    #[serde(flatten)]
    pub workflow: NoticeWorkflow,
}

impl Notice {
    /// Create a notice for `subject`.
    pub fn new(id: NoticeId, subject: impl Into<String>) -> Self {
        let subject: String = subject.into();
        Self {
            id,
            subject: subject.clone(),
            workflow: NoticeWorkflow::new(subject),
        }
    }

    /// The current workflow state.
    pub fn state(&self) -> tpt_c_workflow::NoticeState {
        self.workflow.state()
    }

    /// Convenience: acknowledge the notice.
    pub fn acknowledge(&mut self, at: AuditMeta) {
        let _ = self.workflow.acknowledge(at);
    }

    /// Convenience: resolve the notice.
    pub fn resolve(&mut self, at: AuditMeta) {
        let _ = self.workflow.resolve(at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::AuditMeta;

    #[test]
    fn notice_lifecycle() {
        let mut n = Notice::new(NoticeId::from_uuid(uuid::Uuid::now_v7()), "Non-conformance #3");
        n.acknowledge(AuditMeta::new("owner", "2026-01-01T00:00:00Z"));
        n.resolve(AuditMeta::new("owner", "2026-01-02T00:00:00Z"));
        assert_eq!(n.state(), tpt_c_workflow::NoticeState::Resolved);
    }
}
