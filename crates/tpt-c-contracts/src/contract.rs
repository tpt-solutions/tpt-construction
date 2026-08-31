// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Core contract aggregate: parties, items, and responsibilities.

use serde::{Deserialize, Serialize};
use tpt_c_ids::{ContractId, ContractItemId};
use uuid::Uuid;

use crate::{Amount, ContractError};

/// A party to a contract (owner, contractor, designer, ...).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Party {
    /// Organization / person name.
    pub name: String,
    /// Role on the project (e.g. "General Contractor").
    pub role: String,
}

/// A responsibility assigned to a role under the contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Responsibility {
    /// Role that owns the responsibility.
    pub role: String,
    /// Actor (user / org) assigned.
    pub actor: String,
}

/// Lifecycle status of a contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    /// Being negotiated.
    Draft,
    /// Signed / executed.
    Executed,
    /// In force.
    Active,
    /// Closed / complete.
    Closed,
}

/// A line item within a contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContractItem {
    /// Item identifier.
    pub id: ContractItemId,
    /// Description of the scoped work.
    pub description: String,
    /// Contracted value.
    pub value: Amount,
}

/// A contract aggregate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    /// Contract identifier.
    pub id: ContractId,
    /// Contract name.
    pub name: String,
    /// Parties.
    #[serde(default)]
    pub parties: Vec<Party>,
    /// Contract items.
    #[serde(default)]
    pub items: Vec<ContractItem>,
    /// Assigned responsibilities.
    #[serde(default)]
    pub responsibilities: Vec<Responsibility>,
    /// Status.
    pub status: ContractStatus,
}

impl Contract {
    /// Create a contract (defaults to `Executed`).
    pub fn new(id: ContractId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            parties: Vec::new(),
            items: Vec::new(),
            responsibilities: Vec::new(),
            status: ContractStatus::Executed,
        }
    }

    /// Add a party.
    pub fn add_party(&mut self, name: impl Into<String>, role: impl Into<String>) {
        self.parties.push(Party {
            name: name.into(),
            role: role.into(),
        });
    }

    /// Assign a responsibility.
    pub fn add_responsibility(&mut self, role: impl Into<String>, actor: impl Into<String>) {
        self.responsibilities.push(Responsibility {
            role: role.into(),
            actor: actor.into(),
        });
    }

    /// Add a contract item.
    pub fn add_item(&mut self, description: impl Into<String>, value: Amount) -> ContractItemId {
        let id = ContractItemId::from_uuid(Uuid::now_v7());
        self.items.push(ContractItem {
            id,
            description: description.into(),
            value,
        });
        id
    }

    /// Total contracted value across all items (same currency required).
    pub fn total_value(&self) -> Result<Amount, ContractError> {
        let mut total: Option<Amount> = None;
        for item in &self.items {
            total = Some(match total {
                None => item.value.clone(),
                Some(t) => t.checked_add(item.value.clone())?,
            });
        }
        total.ok_or(ContractError::OutOfRange("no items"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parties_and_items() {
        let mut c = Contract::new(ContractId::from_uuid(Uuid::now_v7()), "Subcontract A");
        c.add_party("Acme Subs", "Subcontractor");
        let id = c.add_item("Formwork", Amount::new(50_000.0, "USD"));
        assert_eq!(c.items.len(), 1);
        assert_eq!(c.items[0].id, id);
        assert_eq!(c.status, ContractStatus::Executed);
    }
}
