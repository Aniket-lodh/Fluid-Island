//! Island state types and transitions.

use std::fmt;

/// The two states of the Fluid Island.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IslandState {
    #[default]
    Collapsed,
    Expanded,
}

impl IslandState {
    /// Flip between `Collapsed` and `Expanded`.
    pub fn toggle(&mut self) {
        *self = match *self {
            Self::Collapsed => Self::Expanded,
            Self::Expanded => Self::Collapsed,
        };
    }

    /// Set the state explicitly.
    #[allow(dead_code)]
    pub fn set(&mut self, state: Self) {
        *self = state;
    }

    /// Return the protocol representation of this state.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Collapsed => "collapsed",
            Self::Expanded => "expanded",
        }
    }
}

impl fmt::Display for IslandState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Collapsed => f.write_str("Collapsed"),
            Self::Expanded => f.write_str("Expanded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_collapsed() {
        assert_eq!(IslandState::default(), IslandState::Collapsed);
    }

    #[test]
    fn toggle_switches_state() {
        let mut state = IslandState::Collapsed;
        state.toggle();
        assert_eq!(state, IslandState::Expanded);
        state.toggle();
        assert_eq!(state, IslandState::Collapsed);
    }

    #[test]
    fn set_replaces_state() {
        let mut state = IslandState::Collapsed;
        state.set(IslandState::Expanded);
        assert_eq!(state, IslandState::Expanded);
        state.set(IslandState::Collapsed);
        assert_eq!(state, IslandState::Collapsed);
    }

    #[test]
    fn display_matches_variant() {
        assert_eq!(IslandState::Collapsed.to_string(), "Collapsed");
        assert_eq!(IslandState::Expanded.to_string(), "Expanded");
    }
}
