// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Approval workflows and state machines for construction field processes.
//!
//! Construction administration is governed by a small set of recurring
//! approval cycles: Requests for Information (RFI), Submittals, Punch List
//! items, Issues, Change Orders, Notices, and Transmittals. Each follows a
//! predictable lifecycle, so rather than hand-rolling ad-hoc status flags this
//! crate provides a reusable [`StateMachine`] plus typed wrappers around the
//! seven canonical workflows.
//!
//! A [`Workflow`] ties a [`StateMachine`] to the routing metadata that makes an
//! approval meaningful: who is assigned to each step, when it is due, and an
//! append-only transition history carrying `AuditMeta` records.

mod routing;
mod state_machine;
mod workflows;

pub use routing::{Assignment, DueDate, Escalation, Role, RoutingError};
pub use state_machine::{StateMachine, StateName, TransitionError};
pub use workflows::{
    ChangeOrderState, ChangeOrderWorkflow, IssueState, IssueWorkflow, NoticeState, NoticeWorkflow,
    PunchListState, PunchListWorkflow, RfiState, RfiWorkflow, SubmittalState, SubmittalWorkflow,
    TransmittalState, TransmittalWorkflow, WorkflowError, WorkflowInstance, WorkflowKind,
};

/// Alias for the generic workflow instance type.
pub type Workflow<S> = WorkflowInstance<S>;

/// Build a fresh [`RfiWorkflow`] for a new request.
///
/// Generates a UUID-backed RFI identifier internally so callers
/// never deal with identifier plumbing.
pub fn new_rfi() -> RfiWorkflow {
    workflows::RfiWorkflow::new()
}
