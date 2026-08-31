// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The seven canonical construction approval workflows.
//!
//! Each workflow is a typed wrapper around a generic [`WorkflowInstance`] that
//! owns a [`StateMachine`]. The wrapper exposes intent-named methods (e.g.
//! [`RfiWorkflow::answer`]) instead of raw state transitions so callers cannot
//! express nonsensical moves; illegal transitions still surface as a
//! [`TransitionError`] from the underlying machine.

use serde::{Deserialize, Serialize};
use tpt_c_core::{AuditMeta, ChangeOrderId, CoreError, RFIId, SubmittalId};
use uuid::Uuid;

use crate::routing::{Assignment, Role};
use crate::state_machine::{StateMachine, StateName, TransitionError};

/// Which flavour of workflow an instance represents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKind {
    /// Request For Information.
    Rfi,
    /// Submittal / shop drawing approval.
    Submittal,
    /// Punch list item.
    PunchList,
    /// Field issue / deficiency.
    Issue,
    /// Change order.
    ChangeOrder,
    /// Notice (e.g. notice of delay, notice of non-conformance).
    Notice,
    /// Transmittal of documents.
    Transmittal,
}

/// A single recorded transition in a workflow's history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionRecord {
    /// State left.
    pub from: String,
    /// State entered.
    pub to: String,
    /// When and by whom.
    pub at: AuditMeta,
}

/// A generic workflow instance over a [`StateName`] enum.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowInstance<S: StateName> {
    /// Stable instance id.
    pub id: Uuid,
    /// Workflow flavour (useful when stored heterogeneously).
    pub kind: WorkflowKind,
    /// The underlying state machine.
    pub machine: StateMachine<S>,
    /// Routing assignments for this workflow.
    #[serde(default)]
    pub assignments: Vec<Assignment>,
    /// Append-only transition history.
    #[serde(default)]
    pub history: Vec<TransitionRecord>,
}

impl<S: StateName> WorkflowInstance<S> {
    /// Create a new instance in `initial` state.
    pub fn new(kind: WorkflowKind, initial: S) -> Self {
        Self {
            id: Uuid::now_v7(),
            kind,
            machine: StateMachine::new(initial),
            assignments: Vec::new(),
            history: Vec::new(),
        }
    }

    /// The current state.
    pub fn current(&self) -> S {
        self.machine.current()
    }

    /// Attempt a transition, recording it in history on success.
    pub fn transition_to(&mut self, to: S, at: AuditMeta) -> Result<(), TransitionError> {
        let from = self.machine.current();
        self.machine.transition(to)?;
        self.history.push(TransitionRecord {
            from: from.state_name().to_string(),
            to: to.state_name().to_string(),
            at,
        });
        Ok(())
    }

    /// Assign a role to an actor for this workflow.
    pub fn assign(&mut self, role: Role, actor: impl Into<String>) {
        self.assignments.push(Assignment::new(role, actor));
    }
}

// ---------------------------------------------------------------------------
// RFI
// ---------------------------------------------------------------------------

/// Lifecycle states of a Request For Information.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RfiState {
    /// Drafted but not yet issued.
    Draft,
    /// Issued and awaiting a response.
    Open,
    /// Answered by the reviewer.
    Answered,
    /// Closed / resolved.
    Closed,
}

impl StateName for RfiState {
    fn state_name(&self) -> &'static str {
        match self {
            RfiState::Draft => "draft",
            RfiState::Open => "open",
            RfiState::Answered => "answered",
            RfiState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            RfiState::Draft => &[RfiState::Open],
            RfiState::Open => &[RfiState::Answered, RfiState::Closed],
            RfiState::Answered => &[RfiState::Closed],
            RfiState::Closed => &[],
        }
    }
}

/// A Request For Information workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RfiWorkflow {
    /// The RFI identifier.
    pub id: RFIId,
    #[serde(flatten)]
    inner: WorkflowInstance<RfiState>,
}

impl RfiWorkflow {
    /// Start a new RFI in `Draft`.
    pub fn new() -> Self {
        Self {
            id: RFIId::from_uuid(Uuid::now_v7()),
            inner: WorkflowInstance::new(WorkflowKind::Rfi, RfiState::Draft),
        }
    }

    /// The current state.
    pub fn state(&self) -> RfiState {
        self.inner.current()
    }

    /// Issue the RFI (Draft -> Open).
    pub fn issue(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(RfiState::Open, at)
    }

    /// Record an answer (Open -> Answered).
    pub fn answer(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(RfiState::Answered, at)
    }

    /// Close the RFI (Answered -> Closed, or Open -> Closed if withdrawn).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(RfiState::Closed, at)
    }

    /// The transition history.
    pub fn history(&self) -> &[TransitionRecord] {
        &self.inner.history
    }
}

impl Default for RfiWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Submittal
// ---------------------------------------------------------------------------

/// Lifecycle states of a Submittal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmittalState {
    /// Drafted, not yet submitted.
    Draft,
    /// Submitted to reviewer.
    Submitted,
    /// Under review.
    UnderReview,
    /// Approved.
    Approved,
    /// Rejected (may be revised and resubmitted).
    Rejected,
    /// Closed.
    Closed,
}

impl StateName for SubmittalState {
    fn state_name(&self) -> &'static str {
        match self {
            SubmittalState::Draft => "draft",
            SubmittalState::Submitted => "submitted",
            SubmittalState::UnderReview => "under_review",
            SubmittalState::Approved => "approved",
            SubmittalState::Rejected => "rejected",
            SubmittalState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            SubmittalState::Draft => &[SubmittalState::Submitted],
            SubmittalState::Submitted => &[SubmittalState::UnderReview],
            SubmittalState::UnderReview => &[SubmittalState::Approved, SubmittalState::Rejected],
            SubmittalState::Rejected => &[SubmittalState::Submitted, SubmittalState::Closed],
            SubmittalState::Approved => &[SubmittalState::Closed],
            SubmittalState::Closed => &[],
        }
    }
}

/// A Submittal approval workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmittalWorkflow {
    /// The submittal identifier.
    pub id: SubmittalId,
    #[serde(flatten)]
    inner: WorkflowInstance<SubmittalState>,
}

impl SubmittalWorkflow {
    /// Start a new submittal in `Draft`.
    pub fn new() -> Self {
        Self {
            id: SubmittalId::from_uuid(Uuid::now_v7()),
            inner: WorkflowInstance::new(WorkflowKind::Submittal, SubmittalState::Draft),
        }
    }

    /// The current state.
    pub fn state(&self) -> SubmittalState {
        self.inner.current()
    }

    /// Submit for review (Draft -> Submitted).
    pub fn submit(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(SubmittalState::Submitted, at)
    }

    /// Begin review (Submitted -> UnderReview).
    pub fn begin_review(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(SubmittalState::UnderReview, at)
    }

    /// Approve (UnderReview -> Approved).
    pub fn approve(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(SubmittalState::Approved, at)
    }

    /// Reject (UnderReview -> Rejected).
    pub fn reject(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(SubmittalState::Rejected, at)
    }

    /// Close (Approved/Rejected -> Closed).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(SubmittalState::Closed, at)
    }
}

impl Default for SubmittalWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Punch List
// ---------------------------------------------------------------------------

/// Lifecycle states of a Punch List item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PunchListState {
    /// Open / identified.
    Open,
    /// Work in progress.
    InProgress,
    /// Resolved by the contractor.
    Resolved,
    /// Verified by the reviewer.
    Verified,
    /// Closed.
    Closed,
}

impl StateName for PunchListState {
    fn state_name(&self) -> &'static str {
        match self {
            PunchListState::Open => "open",
            PunchListState::InProgress => "in_progress",
            PunchListState::Resolved => "resolved",
            PunchListState::Verified => "verified",
            PunchListState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            PunchListState::Open => &[PunchListState::InProgress, PunchListState::Closed],
            PunchListState::InProgress => &[PunchListState::Resolved],
            PunchListState::Resolved => &[PunchListState::Verified],
            PunchListState::Verified => &[PunchListState::Closed],
            PunchListState::Closed => &[],
        }
    }
}

/// A Punch List item workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PunchListWorkflow {
    /// The punch list item identifier.
    pub id: String,
    #[serde(flatten)]
    inner: WorkflowInstance<PunchListState>,
}

impl PunchListWorkflow {
    /// Start a new punch list item in `Open`.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            inner: WorkflowInstance::new(WorkflowKind::PunchList, PunchListState::Open),
        }
    }

    /// The current state.
    pub fn state(&self) -> PunchListState {
        self.inner.current()
    }

    /// Begin work (Open -> InProgress).
    pub fn start(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(PunchListState::InProgress, at)
    }

    /// Mark resolved (InProgress -> Resolved).
    pub fn resolve(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(PunchListState::Resolved, at)
    }

    /// Verify (Resolved -> Verified).
    pub fn verify(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(PunchListState::Verified, at)
    }

    /// Close (Verified -> Closed, or Open -> Closed if voided).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(PunchListState::Closed, at)
    }
}

// ---------------------------------------------------------------------------
// Issue
// ---------------------------------------------------------------------------

/// Lifecycle states of a field Issue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueState {
    /// Open / reported.
    Open,
    /// Assigned to an owner.
    Assigned,
    /// Being worked.
    InProgress,
    /// Resolved.
    Resolved,
    /// Closed.
    Closed,
}

impl StateName for IssueState {
    fn state_name(&self) -> &'static str {
        match self {
            IssueState::Open => "open",
            IssueState::Assigned => "assigned",
            IssueState::InProgress => "in_progress",
            IssueState::Resolved => "resolved",
            IssueState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            IssueState::Open => &[IssueState::Assigned, IssueState::Closed],
            IssueState::Assigned => &[IssueState::InProgress],
            IssueState::InProgress => &[IssueState::Resolved],
            IssueState::Resolved => &[IssueState::Closed],
            IssueState::Closed => &[],
        }
    }
}

/// A field Issue workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueWorkflow {
    /// The issue identifier.
    pub id: String,
    #[serde(flatten)]
    inner: WorkflowInstance<IssueState>,
}

impl IssueWorkflow {
    /// Start a new issue in `Open`.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            inner: WorkflowInstance::new(WorkflowKind::Issue, IssueState::Open),
        }
    }

    /// The current state.
    pub fn state(&self) -> IssueState {
        self.inner.current()
    }

    /// Assign (Open -> Assigned).
    pub fn assign(&mut self, actor: impl Into<String>, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.assign(Role::Approver, actor);
        self.inner.transition_to(IssueState::Assigned, at)
    }

    /// Begin work (Assigned -> InProgress).
    pub fn start(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(IssueState::InProgress, at)
    }

    /// Resolve (InProgress -> Resolved).
    pub fn resolve(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(IssueState::Resolved, at)
    }

    /// Close (Resolved -> Closed, or Open -> Closed).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(IssueState::Closed, at)
    }
}

// ---------------------------------------------------------------------------
// Change Order
// ---------------------------------------------------------------------------

/// Lifecycle states of a Change Order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOrderState {
    /// Drafted.
    Draft,
    /// Pending approval.
    Pending,
    /// Approved.
    Approved,
    /// Rejected (may be revised).
    Rejected,
    /// Executed (work performed).
    Executed,
    /// Closed.
    Closed,
}

impl StateName for ChangeOrderState {
    fn state_name(&self) -> &'static str {
        match self {
            ChangeOrderState::Draft => "draft",
            ChangeOrderState::Pending => "pending",
            ChangeOrderState::Approved => "approved",
            ChangeOrderState::Rejected => "rejected",
            ChangeOrderState::Executed => "executed",
            ChangeOrderState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            ChangeOrderState::Draft => &[ChangeOrderState::Pending],
            ChangeOrderState::Pending => &[ChangeOrderState::Approved, ChangeOrderState::Rejected],
            ChangeOrderState::Rejected => &[ChangeOrderState::Draft, ChangeOrderState::Closed],
            ChangeOrderState::Approved => &[ChangeOrderState::Executed, ChangeOrderState::Closed],
            ChangeOrderState::Executed => &[ChangeOrderState::Closed],
            ChangeOrderState::Closed => &[],
        }
    }
}

/// A Change Order workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeOrderWorkflow {
    /// The change order identifier.
    pub id: ChangeOrderId,
    #[serde(flatten)]
    inner: WorkflowInstance<ChangeOrderState>,
}

impl ChangeOrderWorkflow {
    /// Start a new change order in `Draft`.
    pub fn new() -> Self {
        Self {
            id: ChangeOrderId::from_uuid(Uuid::now_v7()),
            inner: WorkflowInstance::new(WorkflowKind::ChangeOrder, ChangeOrderState::Draft),
        }
    }

    /// The current state.
    pub fn state(&self) -> ChangeOrderState {
        self.inner.current()
    }

    /// Submit for approval (Draft -> Pending).
    pub fn submit(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(ChangeOrderState::Pending, at)
    }

    /// Approve (Pending -> Approved).
    pub fn approve(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(ChangeOrderState::Approved, at)
    }

    /// Reject (Pending -> Rejected).
    pub fn reject(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(ChangeOrderState::Rejected, at)
    }

    /// Revise after rejection (Rejected -> Draft).
    pub fn revise(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(ChangeOrderState::Draft, at)
    }

    /// Mark executed (Approved -> Executed).
    pub fn execute(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(ChangeOrderState::Executed, at)
    }

    /// Close (Approved/Executed -> Closed).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(ChangeOrderState::Closed, at)
    }
}

impl Default for ChangeOrderWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Notice
// ---------------------------------------------------------------------------

/// Lifecycle states of a Notice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeState {
    /// Issued.
    Issued,
    /// Acknowledged by recipient.
    Acknowledged,
    /// Resolved.
    Resolved,
    /// Closed.
    Closed,
}

impl StateName for NoticeState {
    fn state_name(&self) -> &'static str {
        match self {
            NoticeState::Issued => "issued",
            NoticeState::Acknowledged => "acknowledged",
            NoticeState::Resolved => "resolved",
            NoticeState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            NoticeState::Issued => &[NoticeState::Acknowledged, NoticeState::Closed],
            NoticeState::Acknowledged => &[NoticeState::Resolved],
            NoticeState::Resolved => &[NoticeState::Closed],
            NoticeState::Closed => &[],
        }
    }
}

/// A Notice workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeWorkflow {
    /// The notice identifier.
    pub id: String,
    #[serde(flatten)]
    inner: WorkflowInstance<NoticeState>,
}

impl NoticeWorkflow {
    /// Start a new notice in `Issued`.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            inner: WorkflowInstance::new(WorkflowKind::Notice, NoticeState::Issued),
        }
    }

    /// The current state.
    pub fn state(&self) -> NoticeState {
        self.inner.current()
    }

    /// Acknowledge (Issued -> Acknowledged).
    pub fn acknowledge(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(NoticeState::Acknowledged, at)
    }

    /// Resolve (Acknowledged -> Resolved, or Issued -> Resolved).
    pub fn resolve(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(NoticeState::Resolved, at)
    }

    /// Close (Resolved -> Closed, or Issued -> Closed).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(NoticeState::Closed, at)
    }
}

// ---------------------------------------------------------------------------
// Transmittal
// ---------------------------------------------------------------------------

/// Lifecycle states of a Transmittal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransmittalState {
    /// Prepared, not yet sent.
    Prepared,
    /// Sent to recipient.
    Sent,
    /// Received by recipient.
    Received,
    /// Acknowledged.
    Acknowledged,
    /// Closed.
    Closed,
}

impl StateName for TransmittalState {
    fn state_name(&self) -> &'static str {
        match self {
            TransmittalState::Prepared => "prepared",
            TransmittalState::Sent => "sent",
            TransmittalState::Received => "received",
            TransmittalState::Acknowledged => "acknowledged",
            TransmittalState::Closed => "closed",
        }
    }
    fn next_states(&self) -> &'static [Self] {
        match self {
            TransmittalState::Prepared => &[TransmittalState::Sent, TransmittalState::Closed],
            TransmittalState::Sent => &[TransmittalState::Received],
            TransmittalState::Received => &[TransmittalState::Acknowledged],
            TransmittalState::Acknowledged => &[TransmittalState::Closed],
            TransmittalState::Closed => &[],
        }
    }
}

/// A Transmittal workflow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransmittalWorkflow {
    /// The transmittal identifier.
    pub id: String,
    #[serde(flatten)]
    inner: WorkflowInstance<TransmittalState>,
}

impl TransmittalWorkflow {
    /// Start a new transmittal in `Prepared`.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            inner: WorkflowInstance::new(WorkflowKind::Transmittal, TransmittalState::Prepared),
        }
    }

    /// The current state.
    pub fn state(&self) -> TransmittalState {
        self.inner.current()
    }

    /// Send (Prepared -> Sent).
    pub fn send(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(TransmittalState::Sent, at)
    }

    /// Mark received (Sent -> Received).
    pub fn mark_received(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(TransmittalState::Received, at)
    }

    /// Acknowledge (Received -> Acknowledged).
    pub fn acknowledge(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(TransmittalState::Acknowledged, at)
    }

    /// Close (Acknowledged -> Closed, or Prepared -> Closed).
    pub fn close(&mut self, at: AuditMeta) -> Result<(), TransitionError> {
        self.inner.transition_to(TransmittalState::Closed, at)
    }
}

/// Errors raised by workflow operations.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WorkflowError {
    /// A transition was illegal.
    #[error(transparent)]
    Transition(#[from] TransitionError),
    /// A core domain error occurred.
    #[error(transparent)]
    Core(#[from] CoreError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_core::AuditMeta;

    fn at(who: &str) -> AuditMeta {
        AuditMeta::new(who, "2026-01-01T00:00:00Z")
    }

    #[test]
    fn rfi_full_lifecycle() {
        let mut rfi = RfiWorkflow::new();
        assert_eq!(rfi.state(), RfiState::Draft);
        rfi.issue(at("gc")).unwrap();
        rfi.answer(at("arch")).unwrap();
        rfi.close(at("gc")).unwrap();
        assert_eq!(rfi.state(), RfiState::Closed);
        assert_eq!(rfi.history().len(), 3);
    }

    #[test]
    fn rfi_rejects_skipping_answer() {
        let mut rfi = RfiWorkflow::new();
        rfi.issue(at("gc")).unwrap();
        assert!(rfi.close(at("gc")).is_ok());
        // After close, no further transitions allowed.
        assert!(rfi.answer(at("arch")).is_err());
    }

    #[test]
    fn submittal_resubmit_after_reject() {
        let mut s = SubmittalWorkflow::new();
        s.submit(at("gc")).unwrap();
        s.begin_review(at("arch")).unwrap();
        s.reject(at("arch")).unwrap();
        assert_eq!(s.state(), SubmittalState::Rejected);
        s.submit(at("gc")).unwrap();
        assert_eq!(s.state(), SubmittalState::Submitted);
    }

    #[test]
    fn change_order_approved_then_executed() {
        let mut co = ChangeOrderWorkflow::new();
        co.submit(at("gc")).unwrap();
        co.approve(at("owner")).unwrap();
        co.execute(at("gc")).unwrap();
        co.close(at("owner")).unwrap();
        assert_eq!(co.state(), ChangeOrderState::Closed);
    }
}
