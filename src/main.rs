//! This example shows how to send messages between the two cores in the RP2040 chip.
//!
//! The LED on the RP Pico W board is connected differently. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_rp::gpio::{Level, Output, Input, Pull};
use embassy_rp::spi;
use embedded_hal_bus::spi::ExclusiveDevice;
use embassy_time::{Delay, Duration, block_for};
use {defmt_rtt as _, panic_probe as _};

// epd
use epd_waveshare::color::Color;
use epd_waveshare::epd2in13_v2::{Display2in13, Epd2in13};
use epd_waveshare::prelude::{WaveshareDisplay, DisplayRotation};

// embedded graphics
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::image::Image;

use tinybmp::Bmp;

#[cortex_m_rt::entry]
fn main() -> ! {
    let peripherals = embassy_rp::init(Default::default());

    let spi_bus = spi::Spi::new_blocking(
        peripherals.SPI0,
        peripherals.PIN_6, // SCLK
        peripherals.PIN_7, // MOSI
        peripherals.PIN_4, // MISO
        spi::Config::default(),
    );


    let busy_in = Input::new(peripherals.PIN_8, Pull::None);
    let cs = Output::new(peripherals.PIN_2, Level::Low);
    let dc = Output::new(peripherals.PIN_3, Level::Low);
    let reset = Output::new(peripherals.PIN_9, Level::Low);
    
    let mut spi_dev = ExclusiveDevice::new(spi_bus, cs, Delay).unwrap();

    let mut display = Display2in13::default();
    let mut epd = Epd2in13::new(&mut spi_dev, busy_in, dc, reset, &mut Delay, None).unwrap();


    // Clear any existing image
    display.set_rotation(DisplayRotation::Rotate90);
    epd.clear_frame(&mut spi_dev, &mut Delay).unwrap();
    display.clear(Color::White).unwrap();
    epd.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay)
        .unwrap();
    block_for(Duration::from_secs(1));

    draw_text(&mut display, "Littol Junimo!", 3, 100);

    let bmp_data = include_bytes!("../res/junimo.bmp");
    let bmp = Bmp::from_slice(bmp_data).unwrap();
    let image = Image::new(&bmp, Point::new(25, 60));
    image.draw(&mut display).unwrap();

    epd.update_and_display_frame(&mut spi_dev, display.buffer(), &mut Delay)
        .unwrap();
    
    loop {
        info!("Hello world!");
        block_for(Duration::from_secs(1));
    }
}


fn draw_text(display: &mut Display2in13, text: &str, x: i32, y: i32) {
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Color::Black)
        .build();

    Text::with_baseline(text, Point::new(x, y), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
}
