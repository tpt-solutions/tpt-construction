# Changelog

All notable changes to this crate are documented here. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-09-17

### Added

- `StateMachine<S>` finite state machine with `StateMachine::new`, `current`,
  `can_go_to`, and `transition` over the `StateName` trait (`state_name`,
  `next_states`); rejects illegal moves with `TransitionError::Illegal`.
- Seven typed workflows with intent-named transition methods and `AuditMeta`-stamped
  history: `RfiWorkflow` (`RfiState`), `SubmittalWorkflow` (`SubmittalState`),
  `PunchListWorkflow` (`PunchListState`), `IssueWorkflow` (`IssueState`),
  `ChangeOrderWorkflow` (`ChangeOrderState`), `NoticeWorkflow` (`NoticeState`),
  `TransmittalWorkflow` (`TransmittalState`).
- Generic `WorkflowInstance<S>` with `id: Uuid`, `kind: WorkflowKind`, `assignments`,
  and append-only `history: Vec<TransitionRecord>`; `Workflow<S>` type alias.
- Routing metadata: `Assignment` builder (`with_due`, `with_escalation`,
  `satisfied_by`), `AssignmentRecord`, `Role`, `DueDate` with local RFC 3339
  validation, `Escalation`, and `RoutingError`.
- `WorkflowError` (transition + core error wrapper) and convenience `new_rfi()`.
