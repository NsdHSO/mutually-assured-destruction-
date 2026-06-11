#![cfg_attr(not(test), no_std)]

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

/// Boot sequence state machine.
/// Each state represents a boot stage with visual feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootState {
    PowerOn,
    ClockInit,
    PeripheralInit,
    Ready,
    Error(BootError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootError {
    ClockFailure,
    PeripheralFailure,
}

impl BootState {
    pub const fn next(self) -> Self {
        match self {
            BootState::PowerOn => BootState::ClockInit,
            BootState::ClockInit => BootState::PeripheralInit,
            BootState::PeripheralInit => BootState::Ready,
            BootState::Ready => BootState::Ready,
            BootState::Error(_) => BootState::PowerOn,
        }
    }

    pub const fn blink_pattern(self) -> (u32, u32) {
        // (on_ms, off_ms)
        match self {
            BootState::PowerOn => (100, 900),
            BootState::ClockInit => (200, 200),
            BootState::PeripheralInit => (500, 100),
            BootState::Ready => (50, 50),
            BootState::Error(_) => (1000, 0),
        }
    }

    pub const fn enter_error(self, err: BootError) -> Self {
        BootState::Error(err)
    }
}

/// Command protocol state machine for UART/USB CDC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolState {
    Idle,
    ReadingHeader { bytes_received: u8 },
    ReadingPayload { remaining: u16 },
    CheckingCrc,
    Executing,
}

impl ProtocolState {
    pub const fn on_byte(self, byte: u8) -> Self {
        match self {
            ProtocolState::Idle => {
                if byte == 0xAA {
                    ProtocolState::ReadingHeader { bytes_received: 1 }
                } else {
                    ProtocolState::Idle
                }
            }
            ProtocolState::ReadingHeader { bytes_received } => {
                let next = bytes_received + 1;
                if next >= 4 {
                    ProtocolState::ReadingPayload { remaining: 0 }
                } else {
                    ProtocolState::ReadingHeader {
                        bytes_received: next,
                    }
                }
            }
            ProtocolState::ReadingPayload { remaining } => {
                if remaining <= 1 {
                    ProtocolState::CheckingCrc
                } else {
                    ProtocolState::ReadingPayload {
                        remaining: remaining - 1,
                    }
                }
            }
            ProtocolState::CheckingCrc => ProtocolState::Executing,
            ProtocolState::Executing => ProtocolState::Idle,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, ProtocolState::Executing)
    }
}

/// Circular buffer for interrupt-driven UART RX.
pub struct RingBuffer<const N: usize> {
    buf: [u8; N],
    head: usize,
    tail: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, byte: u8) -> Result<(), BufferError> {
        let next = (self.head + 1) % N;
        if next == self.tail {
            return Err(BufferError::Full);
        }
        self.buf[self.head] = byte;
        self.head = next;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            return None;
        }
        let byte = self.buf[self.tail];
        self.tail = (self.tail + 1) % N;
        Some(byte)
    }

    pub const fn len(&self) -> usize {
        (self.head + N - self.tail) % N
    }

    pub const fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub const fn is_full(&self) -> bool {
        (self.head + 1) % N == self.tail
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferError {
    Full,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn led_state_toggle() {
        let s = LedState::new();
        assert!(!s.is_on);
        let s = s.toggle();
        assert!(s.is_on);
        assert_eq!(s.toggle_count, 1);
    }

    #[test]
    fn ring_buffer_basic() {
        let mut buf = RingBuffer::<4>::new();
        assert!(buf.is_empty());
        buf.push(1).unwrap();
        buf.push(2).unwrap();
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), Some(2));
        assert!(buf.is_empty());
    }

    #[test]
    fn ring_buffer_full() {
        let mut buf = RingBuffer::<3>::new();
        buf.push(1).unwrap();
        buf.push(2).unwrap();
        assert!(buf.is_full());
        assert!(buf.push(3).is_err());
    }

    #[test]
    fn protocol_state_transitions() {
        let s = ProtocolState::Idle;
        let s = s.on_byte(0xAA);
        assert!(matches!(s, ProtocolState::ReadingHeader { .. }));
    }
}
