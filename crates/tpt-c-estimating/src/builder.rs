// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The estimate builder: takeoff → cost items → line items → estimate.

use tpt_c_core::ProjectId;
use tpt_c_cost::{CostDatabase, CostItem, Estimate, LineItem, Markup};
use tpt_c_ids::IdFactory;
use tpt_c_model::Project;
use tpt_c_quantities::TakeoffResult;

use crate::code_for;

/// Errors raised while pricing a takeoff.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PricingError {
    /// No cost database was supplied to the builder.
    #[error("no cost database supplied to estimate builder")]
    MissingDatabase,

    /// A quantity's cost code had no rate in the database.
    #[error("no rate for cost code {0}")]
    MissingRate(String),

    /// A line item's currency did not match the estimate currency.
    #[error("currency mismatch pricing estimate: {0}")]
    CurrencyMismatch(String),
}

/// Builds priced [`Estimate`]s from a [`TakeoffResult`].
#[derive(Clone, Debug)]
pub struct EstimateBuilder {
    title: String,
    currency: String,
    project_id: ProjectId,
    database: Option<CostDatabase>,
    markup: Markup,
    /// Price the gross (waste-expanded) quantity when `true`; net when `false`.
    use_gross: bool,
}

impl EstimateBuilder {
    /// A builder for an estimate in `currency`.
    pub fn new(title: impl Into<String>, currency: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            currency: currency.into(),
            project_id: ProjectId::nil(),
            database: None,
            markup: Markup::default(),
            use_gross: true,
        }
    }

    /// Attach a cost database used to resolve unit rates by code.
    pub fn with_database(mut self, db: CostDatabase) -> Self {
        self.database = Some(db);
        self
    }

    /// Override the markup schedule (defaults to overhead 10% / profit 8%).
    pub fn with_markup(mut self, markup: Markup) -> Self {
        self.markup = markup;
        self
    }

    /// Price the gross (waste-expanded) quantity. Call with `false` to price net.
    pub fn price_gross(mut self, use_gross: bool) -> Self {
        self.use_gross = use_gross;
        self
    }

    /// Associate the estimate with a project.
    pub fn for_project(mut self, project: &Project) -> Self {
        self.project_id = project.id;
        self
    }

    /// Run a default takeoff over `project`, then price it.
    pub fn from_project(&self, project: &Project) -> Result<Estimate, PricingError> {
        let takeoff = tpt_c_quantities::TakeoffEngine::new().run(project);
        self.from_takeoff(&takeoff)
    }

    /// Price every quantity in `takeoff`, producing an estimate.
    pub fn from_takeoff(&self, takeoff: &TakeoffResult) -> Result<Estimate, PricingError> {
        let db = self
            .database
            .as_ref()
            .ok_or(PricingError::MissingDatabase)?;
        let mut estimate = Estimate::new(self.title.clone(), self.currency.clone());
        estimate.id = IdFactory::estimate();
        estimate.markup = self.markup;

        let mut number = 0usize;
        for item in &takeoff.items {
            for q in &item.quantities {
                let code = code_for(&item.classification, &item.category);
                let rate = db
                    .get(&code.code)
                    .ok_or_else(|| PricingError::MissingRate(code.code.clone()))?;
                if rate.rate.currency() != self.currency {
                    return Err(PricingError::CurrencyMismatch(
                        rate.rate.currency().to_string(),
                    ));
                }
                let quantity: tpt_c_model::Quantity =
                    if self.use_gross { q.gross() } else { q.net }.into();
                let cost_item = CostItem::new(code, quantity, rate.rate.clone());
                number += 1;
                estimate.add_line(LineItem::from_item(number, &cost_item));
            }
        }
        Ok(estimate)
    }
}
