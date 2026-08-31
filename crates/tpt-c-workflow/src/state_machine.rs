// Copyright (c) TPT Solutions
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic, reusable finite state machine.
//!
//! The machine is intentionally tiny: it holds only the current state and asks
//! the state enum which transitions are legal via [`StateName::next_states`].
//! Because no extra bookkeeping is stored, the machine serializes to a single
//! state value and is trivially reconstructable. Domain workflows (see
//! [`crate::workflows`]) build on this with typed, intent-named methods.

use std::fmt::Debug;
use std::hash::Hash;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A state that can describe itself and enumerate its legal successors.
pub trait StateName: Clone + Copy + PartialEq + Eq + Hash + Debug + Serialize + 'static {
    /// Stable, human-readable name of the state.
    fn state_name(&self) -> &'static str;

    /// The states that may legally follow this one.
    fn next_states(&self) -> &'static [Self];
}

/// Error returned when a transition is attempted.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransitionError {
    /// The requested transition is not allowed from the current state.
    #[error("illegal transition from {from} to {to}")]
    Illegal {
        /// State we are leaving.
        from: &'static str,
        /// State we were asked to enter.
        to: &'static str,
    },
}

/// A finite state machine over a [`StateName`] enum.
///
/// Create it with [`StateMachine::new`] and drive it with
/// [`StateMachine::transition`]. It is cheap to clone and serializes to the
/// single current state value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMachine<S: StateName> {
    current: S,
}

impl<S: StateName> StateMachine<S> {
    /// Create a machine in `initial` state.
    pub fn new(initial: S) -> Self {
        Self { current: initial }
    }

    /// The current state.
    pub fn current(&self) -> S {
        self.current
    }

    /// Whether a transition to `to` is permitted from the current state.
    pub fn can_go_to(&self, to: S) -> bool {
        self.current.next_states().contains(&to)
    }

    /// Attempt to move to `to`, returning a [`TransitionError`] if disallowed.
    pub fn transition(&mut self, to: S) -> Result<(), TransitionError> {
        if self.can_go_to(to) {
            self.current = to;
            Ok(())
        } else {
            Err(TransitionError::Illegal {
                from: self.current.state_name(),
                to: to.state_name(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum TestState {
        A,
        B,
        C,
    }

    impl StateName for TestState {
        fn state_name(&self) -> &'static str {
            match self {
                TestState::A => "a",
                TestState::B => "b",
                TestState::C => "c",
            }
        }
        fn next_states(&self) -> &'static [Self] {
            match self {
                TestState::A => &[TestState::B],
                TestState::B => &[TestState::C],
                TestState::C => &[],
            }
        }
    }

    #[test]
    fn allows_legal_transitions() {
        let mut m = StateMachine::new(TestState::A);
        assert!(m.can_go_to(TestState::B));
        assert!(!m.can_go_to(TestState::C));
        m.transition(TestState::B).unwrap();
        m.transition(TestState::C).unwrap();
        assert_eq!(m.current(), TestState::C);
        assert!(!m.can_go_to(TestState::A));
    }

    #[test]
    fn rejects_illegal_transition() {
        let mut m = StateMachine::new(TestState::A);
        let err = m.transition(TestState::C).unwrap_err();
        assert_eq!(
            err,
            TransitionError::Illegal {
                from: "a",
                to: "c"
            }
        );
    }

    #[test]
    fn roundtrips_through_serde_json() {
        let m = StateMachine::new(TestState::B);
        let json = serde_json::to_string(&m).unwrap();
        assert_eq!(json, "{\"current\":\"b\"}");
        let back: StateMachine<TestState> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, m);
    }
}
