use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus;

/// Number of bytes the Arduino sketch wrote for a full-screen update
pub const ALLSCREEN_GRAPH_BYTES: usize = 4000;

/// Write a full-screen black image to the display and trigger an update.
pub fn write_full_screen<SPI, CS, DC, BUSY>(
    spi: &mut SPI,
    cs: &mut CS,
    dc: &mut DC,
    busy: &mut BUSY,
    brightness: u8,
) where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
{
    // Command 0x24: write RAM (black/white image)
    write_command(spi, cs, dc, 0x24);

    // Write ALLSCREEN_GRAPH_BYTES of 0x00 (black in the Arduino driver)
    for _ in 0..ALLSCREEN_GRAPH_BYTES {
        write_data(spi, cs, dc, brightness);
    }

    // Trigger an update (matches Arduino: 0x22 {0xF7} then 0x20)
    update(spi, cs, dc, busy);
}

fn write_command<SPI, CS, DC>(spi: &mut SPI, cs: &mut CS, dc: &mut DC, cmd: u8)
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
{
    // DC low for command
    let _ = dc.set_low();
    let _ = cs.set_low();
    let _ = spi.write(&[cmd]);
    let _ = cs.set_high();
}

fn write_data<SPI, CS, DC>(spi: &mut SPI, cs: &mut CS, dc: &mut DC, data: u8)
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
{
    // DC high for data
    let _ = dc.set_high();
    let _ = cs.set_low();
    let _ = spi.write(&[data]);
    let _ = cs.set_high();
}

fn update<SPI, CS, DC, BUSY>(spi: &mut SPI, cs: &mut CS, dc: &mut DC, busy: &mut BUSY)
where
    SPI: SpiBus<u8>,
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
