// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Phase 6 integration check: field execution end-to-end.
//!
//! Exercises the daily-log -> RFI -> document -> pay-app chain across the
//! Phase 6 crates so the workflow can be validated as a single scenario. Run
//! with `cargo run --example field-execution-e2e` or
//! `cargo test --example field-execution-e2e`.

use tpt_c_core::{AuditMeta, ProjectId};
use tpt_c_field::{DailyLog, SiteObservation, ObservationCategory, Severity, WeatherCondition, WeatherRecord, WorkRecord};
use tpt_c_ids::{DocumentId, NoticeId, SafetyIncidentId};
use tpt_c_workflow::{RfiWorkflow, RfiState, TransmittalState};
use tpt_c_documents::{DocumentRegister, DocumentType};
use tpt_c_contracts::{Contract, Amount, ClaimKind};
use tpt_c_payapps::{PaymentApplication, PaymentApplicationStatus, ScheduleOfValues};
use tpt_c_cost::Money;
use tpt_c_safety::{Incident, Severity as SafetySeverity, IncidentStatus};

fn at(who: &str) -> AuditMeta {
    AuditMeta::new(who, "2026-03-01T00:00:00Z")
}

fn main() {
    run().expect("phase 6 integration scenario failed");
    println!("phase 6 field-execution-e2e: OK");
}

/// Run the full chain and return an error string on any invariant violation.
fn run() -> Result<(), String> {
    let project = ProjectId::from_uuid(uuid_now());

    // 1. Daily log captures the day on site.
    let log = DailyLog::new(project, "2026-03-01", "foreman");
    let mut log = log.set_weather(WeatherRecord::new(WeatherCondition::Clear, 18.0, 6.0));
    log.add_manpower("carpenters", 4, 8.0);
    log.add_manpower("electricians", 2, 8.0);
    log.add_work(WorkRecord::new("Set rebar, grid A1-C3").with_quantity(12.5, "t"));
    log.add_observation(SiteObservation::new(
        ObservationCategory::Safety,
        "Unguarded floor opening",
        Severity::High,
    ));
    assert_eq!(log.total_man_hours(), 48.0, "man-hours");

    // 2. A deficiency on site raises an RFI (workflow-driven).
    let mut rfi = RfiWorkflow::new();
    rfi.issue(at("foreman")).map_err(|e| e.to_string())?;
    rfi.answer(at("arch")).map_err(|e| e.to_string())?;
    rfi.close(at("foreman")).map_err(|e| e.to_string())?;
    assert_eq!(rfi.state(), RfiState::Closed, "rfi should close");
    log.link_rfi(rfi.id);
    assert_eq!(log.linked_rfis.len(), 1, "rfi linked to log");

    // 3. The RFI response is issued as a transmittal of a document.
    let mut reg = DocumentRegister::new();
    let doc = reg
        .add_document("RFI-001 Response", DocumentType::Report)
        .map_err(|e| e.to_string())?;
    doc_id_check(&doc.id);
    let tx = reg.issue_transmittal("TX-0042", vec![doc.id]);
    let mut tx = tx;
    tx.workflow.send(at("pm")).map_err(|e| e.to_string())?;
    assert_eq!(tx.workflow.state(), TransmittalState::Sent, "transmittal sent");

    // 4. The contract carries a notice and a claim tied to the RFI.
    let mut contract = Contract::new(uuid_contract(), "GC Prime");
    contract.add_item("Concrete works", Amount::new(500_000.0, "USD"));
    let mut notice = tpt_c_contracts::Notice::new(NoticeId::from_uuid(uuid_now()), "Notice of delay");
    notice.acknowledge(at("owner"));
    assert_eq!(notice.state(), tpt_c_workflow::NoticeState::Acknowledged);
    let _claim = tpt_c_contracts::Claim::new(uuid_claim(), ClaimKind::Delay, "Weather delay impact");

    // 5. A payment application draws down against a schedule of values.
    let mut sov = ScheduleOfValues::new("SOV-1");
    sov.add_item("03 30 00", "Cast-in-place concrete", Money::new(500_000.0, "USD"));
    let mut app = PaymentApplication::new(uuid_pa(), "2026-03", sov.total()).with_sov("SOV-1");
    app.set_work_completed(Money::new(120_000.0, "USD"));
    app.set_retainage(0.05).map_err(|e| e.to_string())?;
    app.submit();
    app.begin_review();
    app.approve("owner".to_string());
    app.certify();
    assert_eq!(app.status, PaymentApplicationStatus::Certified, "pay app certified");
    // gross 120k - retainage 6k - prior 0 => 114k certified.
    assert_eq!(app.net_claim().amount(), 114_000.0, "net claim");

    // 6. A safety incident is recorded and investigated.
    let mut incident = Incident::new(SafetyIncidentId::from_uuid(uuid_now()), "Near-miss with forklift", SafetySeverity::Medium);
    incident.begin_investigation();
    assert_eq!(incident.status, IncidentStatus::Investigating);

    Ok(())
}

fn doc_id_check(id: &DocumentId) {
    // touch to ensure type flows through the example
    assert!(!id.as_uuid().is_nil());
}

fn uuid_now() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

fn uuid_contract() -> tpt_c_ids::ContractId {
    tpt_c_ids::ContractId::from_uuid(uuid::Uuid::now_v7())
}

fn uuid_claim() -> tpt_c_ids::ClaimId {
    tpt_c_ids::ClaimId::from_uuid(uuid::Uuid::now_v7())
}

fn uuid_pa() -> tpt_c_ids::PaymentApplicationId {
    tpt_c_ids::PaymentApplicationId::from_uuid(uuid::Uuid::now_v7())
}

#[cfg(test)]
mod tests {
    #[test]
    fn end_to_end_field_workflow() {
        super::run().expect("scenario failed");
    }
}
