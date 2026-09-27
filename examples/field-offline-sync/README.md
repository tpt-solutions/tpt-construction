# field-offline-sync

Phase 7 integration example: two field devices start offline and both edit the
shared equipment register — each keeps its edits as queued CRDT operations
(`tpt-c-sync`) and appends domain events to an append-only audit trail
(`tpt-c-events`). When connectivity returns, the replicas are synchronized;
the LWW-register merge converges deterministically and the audit trail is
replayed into an event-count projection.

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p field-offline-sync
```

The example takes no arguments; it prints:

```text
Offline: device A has 1 queued op(s), device B has 2 queued op(s)
Synced: applied 2 op(s) to A, 1 to B, 2 key(s) converged
Converged excavator state: EquipmentState { name: "Excavator EX-1", status: "idle", hours: 95.0 }
Audit event counts: { <event type>: 1, <event type>: 1 }  (one rfi_created, one payment_application_submitted; HashMap order varies)
field-offline-sync example completed successfully
```

Device B wrote the excavator record last, so its view (`idle`, 95.0 hours) is
the one both replicas converge on.

## What it exercises

- `tpt_c_sync` — `Replica` offline/online transitions, `local_put` queuing,
  pending-operation counts, and the two-way `sync` merge report
  (`applied_to_a`, `applied_to_b`, `converged_keys`).
- `tpt_c_events` — append-only `AuditTrail` of `DomainEvent`s recorded while
  offline, plus `replay` with an `EventTypeCounter` projection.
- `tpt_c_core` / `tpt_c_ids` — `AssetId` and `RFIId` identity types and the
  deterministic `IdFactory`.
- `serde` — the replicated `EquipmentState` value is `Serialize`/
  `Deserialize`, mirroring how state would cross a real network boundary.
- Convergence guarantees: the example asserts both replicas hold identical
  state and have empty pending-op queues after sync.

## License

Dual-licensed `MIT OR Apache-2.0`.
