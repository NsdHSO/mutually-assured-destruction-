#![no_std]

use core::fmt;
use cortex_m::delay::Delay;
use cortex_m::singleton;
use embedded_hal::digital::v2::InputPin;
use rp_pico::hal::clocks::init_clocks_and_plls;
use rp_pico::hal::gpio::{bank0::Gpio25, FunctionSioOutput, Pin, PullDown};
use rp_pico::hal::pac;
use rp_pico::hal::rom_data::reset_to_usb_boot;
use rp_pico::hal::sio::Sio;
use rp_pico::hal::usb::UsbBus;
use rp_pico::hal::watchdog::Watchdog;
use rp_pico::hal::Clock;
use usb_device::class_prelude::*;
use usb_device::device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;

pub use pico_core::LedState;

/// Board initialization failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
    /// Peripherals already taken.
    PeripheralsTaken,
    /// Core peripherals already taken.
    CorePeripheralsTaken,
    /// Clock/PLL initialization failed.
    ClockInitFailed,
    /// USB bus allocator singleton already initialized.
    UsbSingletonFailed,
    /// USB device string descriptors failed.
    UsbStringsFailed,
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InitError::PeripheralsTaken => write!(f, "peripherals already taken"),
            InitError::CorePeripheralsTaken => write!(f, "core peripherals already taken"),
            InitError::ClockInitFailed => write!(f, "clock/PLL init failed"),
            InitError::UsbSingletonFailed => write!(f, "USB singleton already initialized"),
            InitError::UsbStringsFailed => write!(f, "USB string descriptors failed"),
        }
    }
}

/// Reset into USB bootloader mode. Never returns.
pub fn enter_bootloader() -> ! {
    reset_to_usb_boot(0, 0);
    unsafe { core::hint::unreachable_unchecked() }
}

/// Initialized board resources.
pub struct Board {
    pub delay: Delay,
    pub led_pin: Pin<Gpio25, FunctionSioOutput, PullDown>,
    pub serial: SerialPort<'static, UsbBus>,
    pub usb_dev: UsbDevice<'static, UsbBus>,
}

/// Initialize clocks, pins, USB, and bootloader check.
pub fn init() -> Result<Board, InitError> {
    let mut pac = pac::Peripherals::take().ok_or(InitError::PeripheralsTaken)?;
    let core = pac::CorePeripherals::take().ok_or(InitError::CorePeripheralsTaken)?;
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .map_err(|_| InitError::ClockInitFailed)?;

    let mut delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // GP15 = bootloader trigger (connect to GND to enter bootloader)
    let boot_pin = pins.gpio15.into_pull_up_input();
    if boot_pin.is_low().unwrap() {
        delay.delay_ms(100);
        if boot_pin.is_low().unwrap() {
            reset_to_usb_boot(0, 0);
        }
    }

    let led_pin = pins.led.into_push_pull_output();

    // Move USB fields out of pac before singleton! closure to avoid partial-move
    let mut resets = pac.RESETS;
    let usbctrl_regs = pac.USBCTRL_REGS;
    let usbctrl_dpram = pac.USBCTRL_DPRAM;
    let usb_clock = clocks.usb_clock;

    // USB setup — singleton! provides 'static lifetime for allocator
    let usb_bus: &mut UsbBusAllocator<UsbBus> = singleton!(
        : UsbBusAllocator<UsbBus> =
            UsbBusAllocator::new(UsbBus::new(
                usbctrl_regs,
                usbctrl_dpram,
                usb_clock,
                true,
                &mut resets,
            ))
    )
    .ok_or(InitError::UsbSingletonFailed)?;

    let serial = SerialPort::new(usb_bus);

    let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::default()
            .manufacturer("Pico Rust")
            .product("Blink Monitor")
            .serial_number("PICO001")])
        .map_err(|_| InitError::UsbStringsFailed)?
        .device_class(2)
        .build();

    Ok(Board {
        delay,
        led_pin,
        serial,
        usb_dev,
    })
}
