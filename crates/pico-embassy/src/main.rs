#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Ticker};
use pico_core::LedState;
use panic_halt as _;

/// Embassy async blinky + heartbeat.
/// Demonstrates concurrent tasks without blocking.

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);
    let mut led_state = LedState::new();
    let mut ticker = Ticker::every(Duration::from_millis(300));

    loop {
        ticker.next().await;

        led_state = led_state.toggle();

        if led_state.is_on {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
