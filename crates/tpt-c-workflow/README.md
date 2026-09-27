# tpt-c-workflow

Approval workflows and state machines for construction field processes. Construction
administration is governed by a small set of recurring approval cycles — RFIs,
Submittals, Punch List items, Issues, Change Orders, Notices, and Transmittals — so
rather than hand-rolling ad-hoc status flags, this crate provides a reusable
[`StateMachine`] plus typed wrappers around those seven canonical lifecycles, each
with routing assignments, due dates, escalation rules, and an append-only transition
history carrying `AuditMeta`.

Part of the [tpt-construction](../../README.md) workspace.

## Features

- Generic `StateMachine<S>` driven by the `StateName` trait (each state enumerates
  its legal successors); illegal moves return `TransitionError::Illegal`.
- Seven ready-made workflows with intent-named methods: `RfiWorkflow` (`issue` /
  `answer` / `close`), `SubmittalWorkflow` (`submit` / `begin_review` / `approve` /
  `reject`), `PunchListWorkflow`, `IssueWorkflow`, `ChangeOrderWorkflow`
  (`submit` / `approve` / `reject` / `revise` / `execute`), `NoticeWorkflow`, and
  `TransmittalWorkflow`.
- `WorkflowInstance<S>` records an append-only `TransitionRecord` history with
  `AuditMeta` (who + when) for every successful transition.
- `WorkflowKind` tags each instance so heterogeneous workflows can be stored together.
- Routing metadata: `Assignment` (role + actor), `Role` (initiator, reviewer,
  approver, recipient, observer), `DueDate` with local RFC 3339 structural
  validation, and `Escalation` (notify role/actor after N overdue days).
- Convenience constructor `new_rfi()` that hides identifier plumbing.
- Fully serializable (serde) and deterministic — a pure core crate with no IO.

## Usage

```toml
[dependencies]
tpt-c-workflow = "0.1"
```

```rust
use tpt_c_core::AuditMeta;
use tpt_c_workflow::{RfiState, RfiWorkflow};

let at = |who: &str| AuditMeta::new(who, "2026-01-01T00:00:00Z");

let mut rfi = RfiWorkflow::new();
assert_eq!(rfi.state(), RfiState::Draft);
rfi.issue(at("gc")).unwrap();
rfi.answer(at("arch")).unwrap();
rfi.close(at("gc")).unwrap();
assert_eq!(rfi.state(), RfiState::Closed);
assert_eq!(rfi.history().len(), 3);

// Illegal transitions are rejected by the underlying state machine.
let mut submittal = tpt_c_workflow::SubmittalWorkflow::new();
submittal.submit(at("gc")).unwrap();
submittal.begin_review(at("arch")).unwrap();
submittal.reject(at("arch")).unwrap();
assert_eq!(submittal.state(), tpt_c_workflow::SubmittalState::Rejected);
submittal.submit(at("gc")).unwrap(); // resubmit after rejection
assert_eq!(submittal.state(), tpt_c_workflow::SubmittalState::Submitted);
```

## Crate relationships

- **Depends on:** `serde`, `thiserror`, `uuid`, `tpt-c-core` (`AuditMeta`, typed ids).
- **Used by:** `tpt-c-documents` (drives `DocumentTransmittal` with
  `TransmittalWorkflow`), `tpt-c-contracts` (drives `Notice` with `NoticeWorkflow`),
  and the `examples/field-execution-e2e` integration scenario.

## Minimum supported Rust version

Stable Rust 1.82+ (workspace MSRV).

## License

Dual-licensed `MIT OR Apache-2.0` (see [LICENSE-MIT](../../LICENSE-MIT) / [LICENSE-APACHE](../../LICENSE-APACHE)).
