// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A per-project ledger of change orders with approved-impact roll-ups.

use serde::{Deserialize, Serialize};
use tpt_c_core::{ChangeOrderId, ProjectId};
use tpt_c_cost::Money;

use crate::change_order::{ChangeOrder, ChangeOrderStatus};
use crate::ChangeError;

/// All change orders for one project.
///
/// The register owns ordering and uniqueness (`id` and `number` must both be
/// unique) and answers the roll-up questions a project-controls report asks:
/// approved cost impact, approved time impact, and what is still pending.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChangeRegister {
    /// The project this register belongs to.
    pub project_id: ProjectId,
    orders: Vec<ChangeOrder>,
}

impl ChangeRegister {
    /// An empty register for `project_id`.
    pub fn new(project_id: ProjectId) -> Self {
        Self {
            project_id,
            orders: Vec::new(),
        }
    }

    /// Add a change order, rejecting duplicate ids or numbers.
    pub fn add(&mut self, order: ChangeOrder) -> Result<(), ChangeError> {
        if self.orders.iter().any(|o| o.id == order.id) {
            return Err(ChangeError::DuplicateId(order.id.to_string()));
        }
        if self.orders.iter().any(|o| o.number == order.number) {
            return Err(ChangeError::DuplicateNumber(order.number.clone()));
        }
        self.orders.push(order);
        Ok(())
    }

    /// The change order with `id`, if present.
    pub fn get(&self, id: &ChangeOrderId) -> Option<&ChangeOrder> {
        self.orders.iter().find(|o| &o.id == id)
    }

    /// Mutable access to the change order with `id`.
    pub fn get_mut(&mut self, id: &ChangeOrderId) -> Option<&mut ChangeOrder> {
        self.orders.iter_mut().find(|o| &o.id == id)
    }

    /// Transition the order with `id` to `to`.
    pub fn transition(
        &mut self,
        id: &ChangeOrderId,
        to: ChangeOrderStatus,
    ) -> Result<(), ChangeError> {
        self.get_mut(id)
            .ok_or_else(|| ChangeError::UnknownChangeOrder(format!("{id:?}")))?
            .transition(to)
    }

    /// All change orders, in insertion order.
    pub fn orders(&self) -> &[ChangeOrder] {
        &self.orders
    }

    /// Number of change orders in the register.
    pub fn len(&self) -> usize {
        self.orders.len()
    }

    /// Whether the register is empty.
    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    /// Sum of cost impacts over approved (or implemented) orders.
    pub fn approved_cost_impact(&self) -> Result<Money, ChangeError> {
        let mut net = Money::zero("USD");
        for o in self.orders.iter().filter(|o| o.status.is_approved()) {
            net = net.checked_add(o.cost_impact.clone())?;
        }
        Ok(net)
    }

    /// Sum of time impacts (days) over approved (or implemented) orders.
    pub fn approved_time_impact_days(&self) -> f64 {
        self.orders
            .iter()
            .filter(|o| o.status.is_approved())
            .map(|o| o.time_impact_days)
            .sum()
    }

    /// Orders still awaiting a decision (submitted or under review).
    pub fn pending(&self) -> impl Iterator<Item = &ChangeOrder> {
        self.orders.iter().filter(|o| {
            matches!(
                o.status,
                ChangeOrderStatus::Submitted | ChangeOrderStatus::UnderReview
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpt_c_ids::IdFactory;

    fn order(number: &str, cost: f64, days: f64) -> ChangeOrder {
        let id = ChangeOrderId::from_uuid(IdFactory::deterministic(number));
        ChangeOrder::new(id, number, format!("change {number}"))
            .with_cost_impact(Money::new(cost, "USD"))
            .with_time_impact(days)
    }

    #[test]
    fn duplicates_rejected() {
        let mut reg = ChangeRegister::new(ProjectId::nil());
        let first = order("CO-1", 100.0, 1.0);
        reg.add(first).unwrap();
        // Same id, different number: rejected as a duplicate id.
        let mut same_id = order("CO-1", 200.0, 2.0);
        same_id.number = "CO-9".into();
        assert_eq!(
            reg.add(same_id),
            Err(ChangeError::DuplicateId(reg.orders()[0].id.to_string()))
        );
        // Same number, different id: rejected as a duplicate number.
        let mut same_number = order("CO-2", 1.0, 0.0);
        same_number.number = "CO-1".into();
        assert_eq!(
            reg.add(same_number),
            Err(ChangeError::DuplicateNumber("CO-1".into()))
        );
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn approved_rollups_only_count_approved() {
        let mut reg = ChangeRegister::new(ProjectId::nil());
        let mut a = order("CO-1", 10_000.0, 5.0);
        a.transition(ChangeOrderStatus::Submitted).unwrap();
        a.transition(ChangeOrderStatus::UnderReview).unwrap();
        a.transition(ChangeOrderStatus::Approved).unwrap();
        let mut b = order("CO-2", 4_000.0, 2.0);
        b.transition(ChangeOrderStatus::Submitted).unwrap();
        let c = order("CO-3", 999.0, 9.0); // still draft
        reg.add(a).unwrap();
        reg.add(b).unwrap();
        reg.add(c).unwrap();

        assert_eq!(reg.approved_cost_impact().unwrap().amount(), 10_000.0);
        assert_eq!(reg.approved_time_impact_days(), 5.0);
        assert_eq!(reg.pending().count(), 1);

        let id = reg.orders()[1].id;
        reg.transition(&id, ChangeOrderStatus::UnderReview).unwrap();
        assert_eq!(reg.pending().count(), 1);
    }

    #[test]
    fn unknown_transition_target() {
        let mut reg = ChangeRegister::new(ProjectId::nil());
        let id = ChangeOrderId::from_uuid(IdFactory::deterministic("ghost"));
        assert_eq!(
            reg.transition(&id, ChangeOrderStatus::Submitted),
            Err(ChangeError::UnknownChangeOrder(format!("{id:?}")))
        );
    }
}
