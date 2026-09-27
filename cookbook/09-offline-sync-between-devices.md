# 9. Offline sync between devices

> **Crates used:** `tpt-c-sync`, `tpt-c-events`, `tpt-c-ids`, `serde`

Two field devices work with no connectivity: each keeps a local-first replica of shared equipment state (CRDT last-writer-wins registers), takes photos into a grow-only set, and appends domain events to an append-only audit trail. When connectivity returns, a two-way `sync` converges both replicas deterministically — no central coordinator, no lost writes — and the audit log replays into projections.

## Cargo.toml

```toml
[dependencies]
tpt-c-sync = "0.1"
tpt-c-events = "0.1"
tpt-c-ids = "0.1"
serde = { version = "1", features = ["derive"] }
```

## Code

```rust
// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

// Offline sync between two field devices. Each device keeps a local-first
// replica of shared equipment state (a CRDT of LWW registers) and appends to an
// append-only audit trail while disconnected. When connectivity returns, a
// two-way sync converges both replicas deterministically — no central server
// and no lost writes — and the audit log replays into projections.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tpt_c_events::{replay, AuditTrail, DomainEvent, EventTypeCounter};
use tpt_c_ids::{AssetId, IdFactory, RFIId};
use tpt_c_sync::{sync, GSet, Replica};

/// The value each replica stores. It must be serializable so sync can move it
/// between devices as CRDT register payloads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct EquipmentState {
    name: String,
    /// "active" / "idle".
    status: String,
    /// Accumulated engine hours.
    hours: f64,
}

fn main() {
    let excavator = IdFactory::asset();
    let dozer = IdFactory::asset();

    // Two devices, each holding a full local copy of the data it cares about.
    let mut tablet: Replica<AssetId, EquipmentState> = Replica::new("tablet-7");
    let mut phone: Replica<AssetId, EquipmentState> = Replica::new("phone-2");

    // ---------------------------------------------------------------
    // 1. Offline phase: writes always succeed locally and queue up.
    // ---------------------------------------------------------------
    tablet.go_offline();
    phone.go_offline();

    // The tablet provisions the excavator.
    tablet.local_put(
        excavator,
        EquipmentState { name: "Excavator EX-1".into(), status: "active".into(), hours: 120.0 },
    );

    // The phone, unaware of the tablet, edits the same excavator (a true
    // concurrent write) and also provisions the dozer.
    phone.local_put(
        excavator,
        EquipmentState { name: "Excavator EX-1".into(), status: "idle".into(), hours: 95.0 },
    );
    phone.local_put(
        dozer,
        EquipmentState { name: "Dozer DZ-1".into(), status: "active".into(), hours: 40.0 },
    );
    println!(
        "Offline: tablet-7 queued {} op(s), phone-2 queued {} op(s)",
        tablet.pending_ops().len(),
        phone.pending_ops().len()
    );

    // ---------------------------------------------------------------
    // 2. Offline audit: domain events are appended locally, never lost.
    // ---------------------------------------------------------------
    let mut audit: AuditTrail = AuditTrail::new();
    // Deterministic id: the same input string yields the same UUID anywhere.
    let rfi_id = RFIId::from_uuid(IdFactory::deterministic("rfi-1001"));
    audit.append(
        DomainEvent::RFICreated {
            rfi_id,
            subject: "Excavator hydraulic leak".into(),
        },
        "tablet-7",
        "2026-08-31T08:00:00Z",
    );
    audit.append(
        DomainEvent::PaymentApplicationSubmitted {
            application_id: "PA-2026-014".into(),
            amount: 125_000.0,
        },
        "phone-2",
        "2026-08-31T09:30:00Z",
    );
    println!("RFI {rfi_id} raised offline and captured in the local audit trail");

    // ---------------------------------------------------------------
    // 3. Reconnect and sync. The pending queues are exchanged and the
    //    LWW registers merge with a deterministic tie-break (counter,
    //    then actor), so both sides converge on identical state.
    // ---------------------------------------------------------------
    tablet.go_online();
    phone.go_online();
    let report = sync(&mut tablet, &mut phone);
    println!(
        "Sync: {} op(s) applied to tablet-7, {} applied to phone-2, {} key(s) converged",
        report.applied_to_a, report.applied_to_b, report.converged_keys
    );

    // The excavator was written concurrently with equal counters (1 on each
    // device), so the actor tie-break decides: "phone-2" > "tablet-7", and the
    // phone's write wins on BOTH replicas.
    let converged = tablet.get(&excavator).expect("excavator present after sync");
    assert_eq!(tablet.get(&excavator), phone.get(&excavator), "replicas converge");
    println!("Converged excavator state: {converged:?}");

    // ---------------------------------------------------------------
    // 4. A grow-only set CRDT: photos taken offline merge by union, so
    //    nothing is ever lost even when both devices hold the same one.
    // ---------------------------------------------------------------
    let mut photos_a: GSet<String> = GSet::new();
    photos_a.insert("photo-1".into());
    photos_a.insert("photo-2".into());
    let mut photos_b: GSet<String> = GSet::new();
    photos_b.insert("photo-2".into());
    photos_b.insert("photo-3".into());
    photos_a.merge(&photos_b);
    photos_b.merge(&photos_a);
    println!(
        "Photos: {} unique on device A, {} on device B after merge",
        photos_a.len(),
        photos_b.len()
    );

    // ---------------------------------------------------------------
    // 5. Replay the audit trail into projections. The log itself is
    //    append-only; read models are always re-derivable from it.
    // ---------------------------------------------------------------
    let counts = replay(&audit, &EventTypeCounter, HashMap::new());
    println!(
        "Audit replay: {} rfi_created, {} payment_application_submitted over {} event(s)",
        counts.get("rfi_created").copied().unwrap_or(0),
        counts.get("payment_application_submitted").copied().unwrap_or(0),
        audit.len()
    );
}
```

## Run it

```bash
cargo run
```

Expected output (the RFI id is a deterministic UUIDv5 of `"rfi-1001"`; the asset ids are generated but never printed, so the run is stable):

```text
Offline: tablet-7 queued 1 op(s), phone-2 queued 2 op(s)
RFI 9a08877f-e2b1-5bd5-b785-db8e5e2459e4 raised offline and captured in the local audit trail
Sync: 2 op(s) applied to tablet-7, 1 applied to phone-2, 2 key(s) converged
Converged excavator state: EquipmentState { name: "Excavator EX-1", status: "active", hours: 120.0 }
Photos: 3 unique on device A, 3 on device B after merge
Audit replay: 1 rfi_created, 1 payment_application_submitted over 2 event(s)
```

## How it works

1. **Local-first writes.** A `Replica` is a keyed map of `LwwRegister`s plus a pending out-queue. `local_put` stamps the write with a `LwwTimestamp { counter, actor }` (a Lamport clock with an actor tie-break), makes it immediately readable locally, and queues a `SyncOp::Put` — `go_offline` merely stops propagation, never writes.
2. **Deterministic conflict resolution.** Both devices wrote the excavator concurrently. `LwwRegister::merge` keeps the greater timestamp, and `LwwTimestamp` orders by counter first, actor second — so `"phone-2"` beats `"tablet-7"` at the same counter. The same verdict is reached on every replica in any order; that is what makes the merge a CRDT rather than "last save wins".
3. **Two-way sync.** `sync(&mut a, &mut b)` drains both pending queues and merges each into the other, bumping the receiving clock forward (`max`). The `SyncReport` shows traffic in both directions (2 ops into the tablet, 1 into the phone) and how many keys are now held by both sides (2). Queues empty out because `take_pending` moves them.
4. **Add-only data merges by union.** The `GSet` photo set never deletes, so `merge` is set union: both devices end with all 3 photos regardless of merge order. (For data that must support removal, the crate also ships an add-wins `OrSet`.)
5. **Events stay append-only while offline.** `AuditTrail` (an `EventStore<DomainEvent>`) assigns monotonic sequence numbers and wraps each `DomainEvent` in a `StoredEvent` envelope with actor and timestamp. Nothing mutates old entries; `replay` rebuilds read models — here an `EventTypeCounter` — from scratch at any time. In a real deployment you would exchange the raw log batches between devices too (they are plain serializable data), exactly like the register ops.
6. **Reproducible ids.** `IdFactory::deterministic("rfi-1001")` derives the same UUIDv5 everywhere, which is exactly what you want for ids that are minted independently on offline devices and must reconcile later.
