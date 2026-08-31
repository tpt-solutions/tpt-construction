// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Construction schedule and cost risk analysis.
//!
//! This crate quantifies schedule and cost uncertainty for a project network
//! using Monte Carlo simulation. Activities carry optional duration and cost
//! [`DurationDistribution`]/[`CostDistribution`] uncertainty and
//! [`WeatherExposure`]; the [`RiskModel::simulate`] method repeatedly samples
//! those distributions, resolves the network with the [`tpt_c_schedule`] CPM
//! engine, and summarises the resulting project-duration and total-cost
//! distributions ([`SimulationResult`]). From the distributions it derives the
//! probability of meeting a target and the cost contingency required at a given
//! confidence level.
//!
//! The crate is self-contained: it vendors a small PCG64 generator and the
//! probability distributions it needs (see [`prob`]) rather than depending on
//! the optional `tpt-math-prob-dist` substrate (spec §4).

mod model;
mod prob;

pub use model::{
    CostDistribution, DurationDistribution, RiskActivity, RiskError, RiskModel, SimulationResult,
    Statistics, WeatherExposure,
};
pub use prob::Rng;
