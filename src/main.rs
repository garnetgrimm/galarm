#![no_std]
#![no_main]

// Ensure we halt the program on panic
use panic_halt as _;

use adafruit_kb2040 as bsp;
use bsp::hal;

use hal::clocks::Clock;
use hal::fugit::RateExtU32;

// Peripheral Access Crate provides low-level register access
use hal::pac;
mod epd;

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

#[bsp::entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // These are implicitly used by the spi driver if they are in the correct mode
    let spi_mosi = pins.gpio7.into_function::<hal::gpio::FunctionSpi>();
    let spi_miso = pins.gpio4.into_function::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.gpio6.into_function::<hal::gpio::FunctionSpi>();
    let spi = hal::spi::Spi::<_, _, _, 8>::new(pac.SPI0, (spi_mosi, spi_miso, spi_sclk));

    // Exchange the uninitialised SPI driver for an initialised one
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        16.MHz(),
        embedded_hal::spi::MODE_0,
    );

    let mut timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    // Configure simple control pins for an EPD (examples: GPIO8-11).
    // Adjust pins to match your wiring.
    let mut cs = pins.gpio2.into_push_pull_output();
    let mut dc = pins.gpio3.into_push_pull_output();
    let mut rst = pins.gpio9.into_push_pull_output();
    let mut busy = pins.gpio8.into_pull_up_input();

    let _ = rst.set_low();
    timer.delay_ms(10000);
    let _ = rst.set_high();
    timer.delay_ms(10000);

    // Hardware initialization for EPD
    epd::init(&mut spi, &mut cs, &mut dc, &mut busy);

    loop {
        epd::write_full_screen(&mut spi, &mut cs, &mut dc, &mut busy, 0x00);
        timer.delay_ms(1000);
        epd::write_full_screen(&mut spi, &mut cs, &mut dc, &mut busy, 0xFF);
        timer.delay_ms(1000);
    }
}

// End of file
