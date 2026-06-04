#![no_std]

/// Pure LED blink state machine.
/// Tracks on/off state and total toggle count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LedState {
    pub is_on: bool,
    pub toggle_count: u32,
}

impl LedState {
    pub const fn new() -> Self {
        Self {
            is_on: false,
            toggle_count: 0,
        }
    }

    /// Toggle LED. Returns new state (immutable update).
    pub const fn toggle(self) -> Self {
        Self {
            is_on: !self.is_on,
            toggle_count: self.toggle_count.wrapping_add(1),
        }
    }
}

impl Default for LedState {
    fn default() -> Self {
        Self::new()
    }
}
