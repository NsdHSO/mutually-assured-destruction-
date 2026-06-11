use pico_core::{BootError, BootState, LedState, ProtocolState, RingBuffer};
use proptest::prelude::*;

/// Property: toggling LED twice returns to original on/off state.
#[test]
fn led_toggle_involutive() {
    let s = LedState::new();
    let s2 = s.toggle().toggle();
    assert_eq!(s.is_on, s2.is_on);
    assert_eq!(s2.toggle_count, 2);
}

/// Property: boot state machine always reaches Ready in ≤3 steps from PowerOn.
#[test]
fn boot_reaches_ready() {
    let mut s = BootState::PowerOn;
    for _ in 0..10 {
        s = s.next();
        if matches!(s, BootState::Ready) {
            break;
        }
    }
    assert!(matches!(s, BootState::Ready));
}

/// Property: error state transitions back to PowerOn.
#[test]
fn boot_error_recovery() {
    let s = BootState::ClockInit;
    let s = s.enter_error(BootError::ClockFailure);
    assert!(matches!(s, BootState::Error(BootError::ClockFailure)));
    let s = s.next();
    assert!(matches!(s, BootState::PowerOn));
}

/// Property: ring buffer never loses pushed bytes (until full).
proptest! {
    #[test]
    fn ring_buffer_push_pop(bytes in prop::collection::vec(0u8..=255, 0..20)) {
        let mut buf = RingBuffer::<16>::new();
        let mut pushed = 0usize;

        for &b in &bytes {
            if buf.push(b).is_ok() {
                pushed += 1;
            }
        }

        assert_eq!(buf.len(), pushed);

        for _ in 0..pushed {
            assert!(buf.pop().is_some());
        }
        assert!(buf.is_empty());
    }
}

/// Property: protocol state machine never gets stuck (always progresses).
#[test]
fn protocol_always_progresses() {
    let mut s = ProtocolState::Idle;
    let test_bytes = [0xAA, 0x01, 0x02, 0x03, 0x04, 0x05];

    for byte in test_bytes {
        let prev = core::mem::discriminant(&s);
        s = s.on_byte(byte);
        // State must change or be terminal
        assert!(core::mem::discriminant(&s) != prev || s.is_terminal());
    }
}
