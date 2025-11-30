use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus;

/// Number of bytes the Arduino sketch wrote for a full-screen update
pub const EPD_WIDTH: usize = 250;
pub const EPD_HEIGHT: usize = 16;

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
    for _ in 0..(EPD_WIDTH * EPD_HEIGHT) {
        write_data(spi, cs, dc, brightness);
    }

    // Trigger an update (matches Arduino: 0x22 {0xF7} then 0x20)
    update(spi, cs, dc, busy, true);
}

fn write_command<SPI, CS, DC>(spi: &mut SPI, cs: &mut CS, dc: &mut DC, cmd: u8)
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
{
    // DC low for command
    dc.set_low().unwrap();
    cs.set_low().unwrap();
    spi.write(&[cmd]).unwrap();
    cs.set_high().unwrap();
}

fn write_data<SPI, CS, DC>(spi: &mut SPI, cs: &mut CS, dc: &mut DC, data: u8)
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
{
    // DC high for data
    dc.set_high().unwrap();
    cs.set_low().unwrap();
    spi.write(&[data]).unwrap();
    cs.set_high().unwrap();
}

fn update<SPI, CS, DC, BUSY>(
    spi: &mut SPI,
    cs: &mut CS,
    dc: &mut DC,
    busy: &mut BUSY,
    complete: bool,
) where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
{
    write_command(spi, cs, dc, 0x22);
    write_data(spi, cs, dc, if complete { 0xF7 } else { 0xFF });
    write_command(spi, cs, dc, 0x20);

    // Wait for BUSY to go low (Arduino code waits until BUSY==0)
    loop {
        if busy.is_low().unwrap_or(false) {
            break;
        }
    }
}

/// Hardware initialization for the EPD (translated from Arduino EPD_HW_Init)
pub fn init<SPI, CS, DC, BUSY>(spi: &mut SPI, cs: &mut CS, dc: &mut DC, busy: &mut BUSY)
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
{
    // Wait for busy
    while !busy.is_low().unwrap_or(false) {}

    write_command(spi, cs, dc, 0x12); // SWRESET
    while !busy.is_low().unwrap_or(false) {}

    write_command(spi, cs, dc, 0x01); // Driver output control
    write_data(spi, cs, dc, 0xF9);
    write_data(spi, cs, dc, 0x00);
    write_data(spi, cs, dc, 0x00);

    write_command(spi, cs, dc, 0x11); // data entry mode
    write_data(spi, cs, dc, 0x01);

    write_command(spi, cs, dc, 0x44); // set Ram-X address start/end position
    write_data(spi, cs, dc, 0x00);
    write_data(spi, cs, dc, 0x0F); // 0x0F-->(15+1)*8=128

    write_command(spi, cs, dc, 0x45); // set Ram-Y address start/end position
    write_data(spi, cs, dc, 0xF9); // 0xF9-->(249+1)=250
    write_data(spi, cs, dc, 0x00);
    write_data(spi, cs, dc, 0x00);
    write_data(spi, cs, dc, 0x00);

    write_command(spi, cs, dc, 0x3C); // BorderWaveform
    write_data(spi, cs, dc, 0x01);

    write_command(spi, cs, dc, 0x18);
    write_data(spi, cs, dc, 0x80);

    write_command(spi, cs, dc, 0x4E); // set RAM x address count to 0;
    write_data(spi, cs, dc, 0x00);
    write_command(spi, cs, dc, 0x4F); // set RAM y address count to 0X199;
    write_data(spi, cs, dc, 0xF9);
    write_data(spi, cs, dc, 0x00);
    while !busy.is_low().unwrap_or(false) {}
}

macro_rules! get_byte {
    ($val:expr, $n:expr) => {
        (($val >> ($n * 8)) & 0xFF) as u8
    };
}

/// Write a partial image to the display and trigger a partial update (compile-time check for data size).
/// x_start and y_start are pixel coordinates, data is an array, part_column and part_line are region size in pixels.
pub fn write_part<SPI, CS, DC, BUSY, const COLS: usize, const ROWS: usize>(
    spi: &mut SPI,
    cs: &mut CS,
    dc: &mut DC,
    busy: &mut BUSY,
    x_start: u32,
    y_start: u32,
    data: &[[u8; COLS]; ROWS],
) where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    BUSY: InputPin,
{
    let img_rows = ROWS as u32 / 8;
    let img_cols = COLS as u32;

    let x_start = x_start / 8;
    let x_end = x_start + img_rows - 1;

    let y_start = y_start;
    let y_end = y_start + img_cols - 1;

    write_command(spi, cs, dc, 0x44); // set RAM x address start/end
    write_data(spi, cs, dc, get_byte!(x_start, 0));
    write_data(spi, cs, dc, get_byte!(x_end, 0));

    write_command(spi, cs, dc, 0x45); // set RAM y address start/end
    write_data(spi, cs, dc, get_byte!(y_start, 0));
    write_data(spi, cs, dc, get_byte!(y_start, 1));
    write_data(spi, cs, dc, get_byte!(y_end, 0));
    write_data(spi, cs, dc, get_byte!(y_end, 1));

    write_command(spi, cs, dc, 0x4E); // set RAM x address count to x_start
    write_data(spi, cs, dc, get_byte!(x_start, 0));
    write_command(spi, cs, dc, 0x4F); // set RAM y address count to y_start
    write_data(spi, cs, dc, get_byte!(y_start, 0));
    write_data(spi, cs, dc, get_byte!(y_start, 1));

    write_command(spi, cs, dc, 0x24); // Write Black and White image to RAM
    for b in data.iter().flatten() {
        write_data(spi, cs, dc, *b);
    }

    update(spi, cs, dc, busy, false);
}
