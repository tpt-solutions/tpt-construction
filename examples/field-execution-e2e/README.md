# field-execution-e2e

Phase 6 integration example: walks a single field-execution scenario end to
end — a daily site log with weather, manpower, work records, and a safety
observation; a deficiency raised as an RFI through its workflow states; an
RFI response issued as a document transmittal; a contract notice and delay
claim; a payment application drawn against a schedule of values through
submission, review, approval, and certification; and a safety incident moved
into investigation. Any broken invariant fails the run.

Part of the [tpt-construction](../../README.md) workspace.

## Run it

```bash
cargo run -p field-execution-e2e
```

The example takes no arguments. On success it prints:

```text
phase 6 field-execution-e2e: OK
```

If any step of the chain violates an invariant, the scenario returns an error
and the binary exits nonzero.

## What it exercises

- `tpt_c_field` — `DailyLog` with `WeatherRecord`, manpower and `WorkRecord`
  entries (48 total man-hours), `SiteObservation` with severity, and RFI
  linking.
- `tpt_c_workflow` — `RfiWorkflow` (issue -> answer -> close) and transmittal
  and notice state machines.
- `tpt_c_documents` — `DocumentRegister` for registering a document and
  issuing a transmittal.
- `tpt_c_contracts` — `Contract` line items, `Notice` acknowledgement, and a
  delay `Claim`.
- `tpt_c_payapps` — `ScheduleOfValues` and `PaymentApplication` lifecycle
  (submit, review, approve, certify) with 5% retainage and net-claim math.
- `tpt_c_safety` — `Incident` with severity and investigation status.
- `tpt_c_core` / `tpt_c_ids` / `uuid` — audit metadata, project identity, and
  strongly typed IDs (UUIDv7).

The same scenario is also available as a test (`cargo test -p
field-execution-e2e`).

## License

Dual-licensed `MIT OR Apache-2.0`.
