//! This example shows how to send messages between the two cores in the RP2040 chip.
//!
//! The LED on the RP Pico W board is connected differently. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::gpio::{Level, Output, Input, Pull};
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_rp::spi;
use embedded_hal_bus::spi::ExclusiveDevice;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Delay, Timer, Duration};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

// epd
use epd_waveshare::color::Color;
use epd_waveshare::epd1in54_v2::{Display1in54, Epd1in54};
use epd_waveshare::prelude::WaveshareDisplay;

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static CHANNEL: Channel<CriticalSectionRawMutex, LedState, 1> = Channel::new();

enum LedState {
    On,
    Off,
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = embassy_rp::init(Default::default());
    let led = Output::new(peripherals.PIN_26, Level::Low);

    let spi_bus = spi::Spi::new_blocking(
        peripherals.SPI1,
        peripherals.PIN_10,
        peripherals.PIN_11,
        peripherals.PIN_12,
        spi::Config::default(),
    );


    let busy_in = Input::new(peripherals.PIN_1, Pull::None);
    let cs = Output::new(peripherals.PIN_2, Level::Low);
    let dc = Output::new(peripherals.PIN_3, Level::Low);
    let reset = Output::new(peripherals.PIN_4, Level::Low);
    
    let mut spi_dev = ExclusiveDevice::new(spi_bus, cs, Delay).unwrap();

    let mut display = Display1in54::default();
    let mut epd = Epd1in54::new(&mut spi_dev, busy_in, dc, reset, &mut Delay, None).unwrap();


    // Clear any existing image
    epd.clear_frame(&mut spi_dev, &mut Delay).unwrap();
    epd.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay)
        .unwrap();
    let _ = Timer::after(Duration::from_secs(5));
    loop {}
}