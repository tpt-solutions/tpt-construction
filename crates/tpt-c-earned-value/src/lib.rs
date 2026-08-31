// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Earned Value Management (EVM) metrics for project cost/schedule control.
//!
//! Implements the core EVM indicators (SPI, CPI, SV, CV, TCPI, EAC, ETC) from
//! the planned/earned/actual baselines. All monetary values use the project
//! currency; all are plain `f64` so they compose with [`tpt_c_cost`].

/// Earned value inputs for a control period.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EarnedValue {
    /// Budgeted Cost of Work Scheduled (planned value, BCWS).
    pub bcws: f64,
    /// Budgeted Cost of Work Performed (earned value, BCWP).
    pub bcwp: f64,
    /// Actual Cost of Work Performed (ACWP).
    pub acwp: f64,
    /// Budget At Completion (BAC).
    pub bac: f64,
}

impl EarnedValue {
    /// Build an earned value snapshot.
    pub fn new(bcws: f64, bcwp: f64, acwp: f64, bac: f64) -> Self {
        Self {
            bcws,
            bcwp,
            acwp,
            bac,
        }
    }

    /// Schedule Performance Index (`BCWP / BCWS`).
    pub fn spi(self) -> f64 {
        safe_div(self.bcwp, self.bcws)
    }

    /// Cost Performance Index (`BCWP / ACWP`).
    pub fn cpi(self) -> f64 {
        safe_div(self.bcwp, self.acwp)
    }

    /// Schedule Variance (`BCWP - BCWS`).
    pub fn sv(self) -> f64 {
        self.bcwp - self.bcws
    }

    /// Cost Variance (`BCWP - ACWP`).
    pub fn cv(self) -> f64 {
        self.bcwp - self.acwp
    }

    /// Estimate At Completion using the CPI-based forecast: `BAC / CPI`.
    pub fn eac(self) -> f64 {
        safe_div(self.bac, self.cpi())
    }

    /// Estimate To Complete (`EAC - ACWP`).
    pub fn etc(self) -> f64 {
        self.eac() - self.acwp
    }

    /// To-Complete Performance Index: `(BAC - BCWP) / (BAC - ACWP)`.
    ///
    /// The CPI that must be achieved on remaining work to meet the BAC.
    pub fn tcpip(self) -> f64 {
        safe_div(self.bac - self.bcwp, self.bac - self.acwp)
    }

    /// Variance At Completion (`BAC - EAC`).
    pub fn vac(self) -> f64 {
        self.bac - self.eac()
    }
}

fn safe_div(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        f64::NAN
    } else {
        a / b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_indicators() {
        let ev = EarnedValue::new(100.0, 80.0, 90.0, 200.0);
        assert!((ev.spi() - 0.8).abs() < 1e-9);
        assert!((ev.cpi() - 80.0 / 90.0).abs() < 1e-9);
        assert!((ev.sv() - -20.0).abs() < 1e-9);
        assert!((ev.cv() - -10.0).abs() < 1e-9);
        assert!((ev.eac() - 200.0 / (80.0 / 90.0)).abs() < 1e-9);
        assert!((ev.tcpip() - 120.0 / 110.0).abs() < 1e-9);
    }

    #[test]
    fn zero_baseline_is_nan_safe() {
        let ev = EarnedValue::new(0.0, 0.0, 0.0, 100.0);
        assert!(ev.spi().is_nan());
    }
}
