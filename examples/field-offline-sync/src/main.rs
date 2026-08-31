// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Field offline-sync integration example (Phase 7 integration check).
//!
//! Two field devices start disconnected. Each records local edits to the shared
//! equipment register (via CRDTs from `tpt-c-sync`) and appends field events to
//! a local audit log (via `tpt-c-events`). When connectivity is restored the two
//! replicas are synchronized; the CRDT merge converges deterministically and the
//! audit trail remains append-only and replayable.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tpt_c_core::{AssetId, RFIId};
use tpt_c_events::{AuditTrail, DomainEvent, EventTypeCounter, replay};
use tpt_c_ids::IdFactory;
use tpt_c_sync::{Replica, sync};

/// A small snapshot of equipment state replicated across field devices.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct EquipmentState {
    name: String,
    /// Operating status string (e.g. "active", "idle").
    status: String,
    /// Accumulated engine hours.
    hours: f64,
}

fn main() {
    let exc = IdFactory::asset();
    let dozer = IdFactory::asset();

    let mut device_a: Replica<AssetId, EquipmentState> = Replica::new("device-a");
    let mut device_b: Replica<AssetId, EquipmentState> = Replica::new("device-b");

    // --- Offline phase -------------------------------------------------------
    device_a.go_offline();
    device_b.go_offline();

    // Device A provisions the excavator and logs some hours.
    device_a.local_put(
        exc,
        EquipmentState {
            name: "Excavator EX-1".into(),
            status: "active".into(),
            hours: 120.0,
        },
    );

    // Device B, unaware of A, provisions the same excavator with different
    // hours and also the dozer.
    device_b.local_put(
        exc,
        EquipmentState {
            name: "Excavator EX-1".into(),
            status: "idle".into(),
            hours: 95.0,
        },
    );
    device_b.local_put(
        dozer,
        EquipmentState {
            name: "Dozer DZ-1".into(),
            status: "active".into(),
            hours: 40.0,
        },
    );

    assert_eq!(device_a.pending_ops().len(), 1);
    assert_eq!(device_b.pending_ops().len(), 2);
    println!("Offline: device A has {} queued op(s), device B has {} queued op(s)",
        device_a.pending_ops().len(),
        device_b.pending_ops().len());

    // --- Local audit trail (append-only, even while offline) -----------------
    let mut audit: AuditTrail = AuditTrail::new();
    audit.append(
        DomainEvent::RFICreated {
            rfi_id: RFIId::from_uuid(IdFactory::deterministic("rfi-offline-1")),
            subject: "Excavator hydraulic leak".into(),
        },
        "device-a",
        "2026-08-31T08:00:00Z",
    );
    audit.append(
        DomainEvent::PaymentApplicationSubmitted {
            application_id: "PA-2026-014".into(),
            amount: 125_000.0,
        },
        "device-b",
        "2026-08-31T09:30:00Z",
    );

    // --- Reconnect + sync ----------------------------------------------------
    device_a.go_online();
    device_b.go_online();
    let report = sync(&mut device_a, &mut device_b);

    // Both replicas converge. For the shared key, the greater LWW timestamp wins;
    // here device B wrote last, so its view is the convergent one.
    let a_exc = device_a.get(&exc).expect("excavator present on A after sync");
    let b_exc = device_b.get(&exc).expect("excavator present on B after sync");
    assert_eq!(a_exc, b_exc, "replicas must converge");
    assert_eq!(device_a.get(&dozer), device_b.get(&dozer));
    assert!(device_a.pending_ops().is_empty());
    assert!(device_b.pending_ops().is_empty());

    println!(
        "Synced: applied {} op(s) to A, {} to B, {} key(s) converged",
        report.applied_to_a, report.applied_to_b, report.converged_keys
    );
    println!("Converged excavator state: {:?}", a_exc);

    // --- Replay the audit trail into a projection ----------------------------
    let counts = replay(&audit, &EventTypeCounter, HashMap::new());
    println!("Audit event counts: {:?}", counts);
    assert_eq!(counts.get("rfi_created"), Some(&1));
    assert_eq!(counts.get("payment_application_submitted"), Some(&1));

    println!("field-offline-sync example completed successfully");
}
