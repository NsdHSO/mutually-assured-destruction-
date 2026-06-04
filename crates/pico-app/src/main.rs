#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;
use pico_setup::{enter_bootloader, init, Board, LedState};

#[entry]
fn main() -> ! {
    let Board {
        mut delay,
        mut led_pin,
        mut serial,
        mut usb_dev,
    } = init().expect("board init failed");

    let mut led_state = LedState::new();
    let mut serial_buf = [0u8; 64];
    let mut last_blink_ms: u32 = 0;
    let mut elapsed_ms: u32 = 0;

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            match serial.read(&mut serial_buf) {
                Ok(count) if count > 0 => {
                    for byte in &serial_buf[..count] {
                        match *byte {
                            b'b' => {
                                let _ = serial.write(b"Entering bootloader...\r\n");
                                delay.delay_ms(50);
                                enter_bootloader();
                            }
                            b'h' | b'?' => {
                                let _ = serial.write(b"Commands: b=bootloader, h=help\r\n");
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        elapsed_ms += 1;
        if elapsed_ms - last_blink_ms >= 300 {
            last_blink_ms = elapsed_ms;
            led_state = led_state.toggle();

            if led_state.is_on {
                led_pin.set_high().unwrap();
                delay.delay_ms(50);
                led_pin.set_low().unwrap();
                delay.delay_ms(75);
                led_pin.set_high().unwrap();
            } else {
                led_pin.set_low().unwrap();
            }

            let mut msg = heapless::String::<64>::new();
            let _ = write!(
                &mut msg,
                "LED: {} | Toggles: {}\r\n",
                if led_state.is_on { "ON " } else { "OFF" },
                led_state.toggle_count
            );
            let _ = serial.write(msg.as_bytes());
        }

        delay.delay_ms(1);
    }
}
