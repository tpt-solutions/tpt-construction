# 7. RFI approval workflow

> **Crates used:** `tpt-c-workflow`, `tpt-c-field`, `tpt-c-core`, `tpt-c-ids`

Turn a deficiency spotted on the morning daily log into a Request For Information, drive it through a guarded approval state machine, route it to the reviewer with a due date and escalation rule, and link it back to the field record. Because every step is a domain event in an append-only transition history, the approval trail survives audits and disputes.

## Cargo.toml

```toml
[dependencies]
tpt-c-workflow = "0.1"
tpt-c-field = "0.1"
tpt-c-core = "0.1"
tpt-c-ids = "0.1"
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

// RFI approval workflow: a quality deficiency spotted on the morning daily log
// is escalated into a Request For Information, guarded by a state machine,
// routed to the architect with a due date and escalation rule, and linked back
// to the log — leaving a complete, auditable transition history.

use tpt_c_core::{AuditMeta, ProjectId};
use tpt_c_field::{
    DailyLog, ObservationCategory, Severity, SiteObservation, WeatherCondition, WeatherRecord,
    WorkRecord,
};
use tpt_c_ids::{IdFactory, RFIId};
use tpt_c_workflow::{
    Assignment, DueDate, Escalation, Role, RfiState, RfiWorkflow, WorkflowInstance, WorkflowKind,
};

/// Every workflow transition is stamped with who acted and when (RFC 3339).
fn at(who: &str, when: &str) -> AuditMeta {
    AuditMeta::new(who, when)
}

fn main() {
    // -------------------------------------------------------------------------
    // 1. The daily log: weather, crews, work performed, and the deficiency.
    // -------------------------------------------------------------------------
    let project = ProjectId::from_uuid(IdFactory::deterministic("project-rivergate"));
    let mut log = DailyLog::new(project, "2026-03-01", "foreman")
        .set_weather(WeatherRecord::new(WeatherCondition::Clear, 18.0, 6.0));
    log.add_manpower("carpenters", 4, 8.0);
    log.add_manpower("electricians", 2, 8.0);
    log.add_work(WorkRecord::new("Set rebar, grid A1-C3").with_quantity(12.5, "t"));
    log.add_observation(SiteObservation::new(
        ObservationCategory::Quality,
        "Rebar spacing does not match the placement drawing",
        Severity::High,
    ));
    println!(
        "Daily log {}: {} trades, {:.1} man-hours, {} open observation(s)",
        log.date,
        log.trade_count(),
        log.total_man_hours(),
        log.open_observations()
    );

    // -------------------------------------------------------------------------
    // 2. Escalate the observation into an RFI and drive the state machine.
    // -------------------------------------------------------------------------
    // `RfiWorkflow::new()` starts in `Draft` and mints a random UUIDv7 id; swap
    // in a deterministic one so the printed output below is stable (in
    // production you would keep the minted id).
    let mut rfi = RfiWorkflow::new();
    rfi.id = RFIId::from_uuid(IdFactory::deterministic("rfi-0042"));

    // Guard rail: an RFI cannot be answered while it is still a draft. The
    // state machine rejects the move instead of leaving a hole in the audit.
    if let Err(e) = rfi.answer(at("architect", "2026-03-02T09:00:00Z")) {
        println!("Guard rail: {e}");
    }

    // Legal lifecycle: Draft -> Open -> Answered -> Closed. Each step is
    // intent-named, refuses out-of-order calls, and appends to the history.
    rfi.issue(at("foreman", "2026-03-01T08:00:00Z"))
        .expect("draft -> open");
    rfi.answer(at("architect", "2026-03-05T14:30:00Z"))
        .expect("open -> answered");
    rfi.close(at("foreman", "2026-03-06T07:45:00Z"))
        .expect("answered -> closed");

    // Tie the RFI back to the field record that spawned it.
    log.link_rfi(rfi.id);

    println!("RFI {} final state: {:?}", rfi.id, rfi.state());
    for step in rfi.history() {
        println!(
            "  {} -> {} (by {} at {})",
            step.from, step.to, step.at.actor, step.at.at
        );
    }
    println!("Daily log now links {} RFI(s)", log.linked_rfis.len());

    // -------------------------------------------------------------------------
    // 3. Routing: who must act, by when, and what happens when they don't.
    // -------------------------------------------------------------------------
    // Assignments hang off the generic WorkflowInstance engine that every typed
    // workflow (RfiWorkflow included) wraps. A service would persist this
    // alongside the RFI to power inboxes, reminders and escalations.
    let mut routed = WorkflowInstance::new(WorkflowKind::Rfi, RfiState::Open);
    routed.assign(Role::Initiator, "foreman");
    routed.assignments.push(
        Assignment::new(Role::Reviewer, "architect@design.example")
            .with_due(DueDate::new("2026-03-08T17:00:00Z"))
            .with_escalation(Escalation {
                notify: Role::Approver,
                actor: "pm@build.example".to_string(),
                after_days: 2,
            }),
    );
    println!("Routing for RFI in state {:?}:", routed.current());
    for a in &routed.assignments {
        let due = a.due.as_ref().map(|d| d.0.as_str()).unwrap_or("-");
        let escalation = match a.escalation.as_ref() {
            Some(esc) => format!("escalates to {} after {} day(s)", esc.actor, esc.after_days),
            None => "no escalation".to_string(),
        };
        println!("  {:?} {} due {due} ({escalation})", a.role, a.actor);
    }

    // When the reviewer finally acts, mint a record that the assignment was
    // satisfied — useful for SLA reporting — and advance the machine.
    let reviewer = &routed.assignments[1];
    let record = reviewer.satisfied_by(at("architect", "2026-03-05T14:30:00Z"));
    println!(
        "Assignment satisfied: {:?} by {} at {}",
        record.role, record.actor, record.at.at
    );
    routed
        .transition_to(RfiState::Answered, at("architect", "2026-03-05T14:30:00Z"))
        .expect("open -> answered");
    println!("Routed RFI now in state {:?}", routed.current());
}
```

## Run it

```bash
cargo run
```

Expected output (the RFI id is a deterministic UUIDv5 of `"rfi-0042"` in the TPT namespace, so the run is reproducible):

```text
Daily log 2026-03-01: 2 trades, 48.0 man-hours, 1 open observation(s)
Guard rail: illegal transition from draft to answered
RFI 5912caa7-11a7-5d1e-88c5-5ac281f374a1 final state: Closed
  draft -> open (by foreman at 2026-03-01T08:00:00Z)
  open -> answered (by architect at 2026-03-05T14:30:00Z)
  answered -> closed (by foreman at 2026-03-06T07:45:00Z)
Daily log now links 1 RFI(s)
Routing for RFI in state Open:
  Initiator foreman due - (no escalation)
  Reviewer architect@design.example due 2026-03-08T17:00:00Z (escalates to pm@build.example after 2 day(s))
Assignment satisfied: Reviewer by architect@design.example at 2026-03-05T14:30:00Z
Routed RFI now in state Answered
```

## How it works

1. **Daily log context.** `DailyLog::new(project, date, created_by)` opens the site record; `set_weather`, `add_manpower`, `add_work` and `add_observation` populate it. The aggregates answer the manager questions directly: `total_man_hours()` (4 × 8 + 2 × 8 = 48), `trade_count()` and `open_observations()`.
2. **Typed lifecycle, not status flags.** `RfiWorkflow::new()` starts in `RfiState::Draft`. Each move (`issue`, `answer`, `close`) is an intent-named method over an internal `StateMachine`, so an out-of-order call — answering a draft — returns `TransitionError::Illegal` with the exact states involved instead of corrupting the record.
3. **Audit for free.** Every successful transition appends a `TransitionRecord` (from-state, to-state, `AuditMeta` with actor and RFC 3339 timestamp) to an append-only `history`. `log.link_rfi(rfi.id)` closes the loop from the field record to the workflow.
4. **Routing metadata.** Role/actor assignments live on the generic `WorkflowInstance` engine that all seven canonical workflows (RFI, Submittal, Punch List, Issue, Change Order, Notice, Transmittal) wrap. `Assignment::new(role, actor)` builds fluently with `.with_due(DueDate)` — validated as RFC 3339 via `DueDate::parse` — and `.with_escalation(...)` describing who is notified after how many overdue days. `Assignment::satisfied_by(meta)` mints the proof that a step was completed, feeding SLA reporting.
5. **Same engine underneath.** `routed.transition_to(RfiState::Answered, ...)` is the exact call `RfiWorkflow::answer` makes internally — the typed wrappers are thin, so you can drop to `WorkflowInstance<S>` whenever you need the routing fields, and drop back without changing storage.
6. **Deterministic ids.** `IdFactory::deterministic(name)` derives a UUIDv5 in the TPT namespace, so the printed RFI id is identical on every run — handy for tests, fixtures and this cookbook. In production, keep the minted UUIDv7 ids: they are unique and time-ordered.
