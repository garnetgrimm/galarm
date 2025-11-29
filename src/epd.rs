use adafruit_kb2040::hal::spi::{Spi, SpiDevice, State, ValidSpiPinout};
use embedded_hal::digital::{InputPin, OutputPin};

/// Number of bytes the Arduino sketch wrote for a full-screen update
pub const ALLSCREEN_GRAPH_BYTES: usize = 4000;

/// Write a full-screen black image to the display and trigger an update.
///
/// This is a minimal port of the Arduino `EPD_WhiteScreen_Black` routine.
/// It assumes the SPI device and the control pins are already configured.
pub fn epd_white_screen_black<S, D, P, CS, DC, BUSY>(
    spi: &mut Spi<S, D, P>,
    cs: &mut CS,
    dc: &mut DC,
    busy: &mut BUSY,
) where
    // use the SPI bus write trait from embedded-hal 1.0 so we can call `spi.write(&[u8])`
    S: State,
    D: SpiDevice,
    P: ValidSpiPinout<D>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
{
    // Command 0x24: write RAM (black/white image)
    write_command(spi, cs, dc, 0x24);

    // Write ALLSCREEN_GRAPH_BYTES of 0x00 (black in the Arduino driver)
    for _ in 0..ALLSCREEN_GRAPH_BYTES {
        write_data(spi, cs, dc, 0x00);
    }

    // Trigger an update (matches Arduino: 0x22 {0xF7} then 0x20)
    update(spi, cs, dc, busy);
}

fn write_command<S, D, P, CS, DC>(spi: &mut Spi<S, D, P>, cs: &mut CS, dc: &mut DC, cmd: u8)
where
    S: State,
    D: SpiDevice,
    P: ValidSpiPinout<D>,
    CS: OutputPin,
    DC: OutputPin,
    CS: OutputPin,
    DC: OutputPin,
{
    // DC low for command
    dc.set_low();
    cs.set_low();
    spi.write(&[cmd]);
    cs.set_high();
}

fn write_data<S, D, P, CS, DC>(spi: &mut Spi<S, D, P>, cs: &mut CS, dc: &mut DC, data: u8)
where
    S: State,
    D: SpiDevice,
    P: ValidSpiPinout<D>,
    CS: OutputPin,
    DC: OutputPin,
{
    // DC high for data
    dc.set_high();
    cs.set_low();
    spi.write(&[data]);
    cs.set_high();
}

fn update<S, D, P, CS, DC, BUSY>(spi: &mut Spi<S, D, P>, cs: &mut CS, dc: &mut DC, busy: &mut BUSY)
where
    S: State,
    D: SpiDevice,
    P: ValidSpiPinout<D>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
{
    write_command(spi, cs, dc, 0x22);
    write_data(spi, cs, dc, 0xF7);
    write_command(spi, cs, dc, 0x20);

    // Wait for BUSY to go low (Arduino code waits until BUSY==0)
    loop {
        if busy.is_low().unwrap_or(false) {
            break;
        }
    }
}
