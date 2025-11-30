use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus;

pub const EPD_WIDTH: usize = 250;
pub const EPD_HEIGHT: usize = 16;

pub struct PaperDisplay<SPI, CS, DC, RST, BUSY> {
    pub spi: SPI,
    pub cs: CS,
    pub dc: DC,
    pub rst: RST,
    pub busy: BUSY,
}

macro_rules! get_byte {
    ($val:expr, $n:expr) => {
        (($val >> ($n * 8)) & 0xFF) as u8
    };
}

impl<SPI, CS, DC, RST, BUSY> PaperDisplay<SPI, CS, DC, RST, BUSY>
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    /// Hardware initialization for the EPD (translated from Arduino EPD_HW_Init)
    pub fn init(&mut self, delay: &mut impl embedded_hal::delay::DelayNs) {
        // Reset sequence
        let _ = self.rst.set_low();
        delay.delay_ms(1);
        let _ = self.rst.set_high();
        delay.delay_ms(1);

        // Wait for busy
        while !self.busy.is_low().unwrap_or(false) {}

        self.write_command(0x12); // SWRESET
        while !self.busy.is_low().unwrap_or(false) {}

        self.write_command(0x01); // Driver output control
        self.write_data(0xF9);
        self.write_data(0x00);
        self.write_data(0x00);

        self.write_command(0x11); // data entry mode
        self.write_data(0x01);

        self.write_command(0x44); // set Ram-X address start/end position
        self.write_data(0x00);
        self.write_data(0x0F); // 0x0F-->(15+1)*8=128

        self.write_command(0x45); // set Ram-Y address start/end position
        self.write_data(0xF9); // 0xF9-->(249+1)=250
        self.write_data(0x00);
        self.write_data(0x00);
        self.write_data(0x00);

        self.write_command(0x3C); // BorderWaveform
        self.write_data(0x01);

        self.write_command(0x18);
        self.write_data(0x80);

        self.write_command(0x4E); // set RAM x address count to 0;
        self.write_data(0x00);
        self.write_command(0x4F); // set RAM y address count to 0X199;
        self.write_data(0xF9);
        self.write_data(0x00);
        while !self.busy.is_low().unwrap_or(false) {}
    }

    /// Write a full-screen black image to the display and trigger an update.
    pub fn write_full_screen(&mut self, brightness: u8) {
        // Command 0x24: write RAM (black/white image)
        self.write_command(0x24);

        // Write ALLSCREEN_GRAPH_BYTES of 0x00 (black in the Arduino driver)
        for _ in 0..(EPD_WIDTH * EPD_HEIGHT) {
            self.write_data(brightness);
        }

        // Trigger an update (matches Arduino: 0x22 {0xF7} then 0x20)
        self.update(true);
    }

    /// Write a partial image to the display and trigger a partial update (compile-time check for data size).
    /// x_start and y_start are pixel coordinates, data is an array, part_column and part_line are region size in pixels.
    pub fn write_part<const COLS: usize, const ROWS: usize>(
        &mut self,
        x_start: u32,
        y_start: u32,
        data: &[[u8; COLS]; ROWS],
    ) {
        let img_rows = ROWS as u32 / 8;
        let img_cols = COLS as u32;

        let x_start = x_start / 8;
        let x_end = x_start + img_rows - 1;

        let y_start = y_start;
        let y_end = y_start + img_cols - 1;

        self.write_command(0x44); // set RAM x address start/end
        self.write_data(get_byte!(x_start, 0));
        self.write_data(get_byte!(x_end, 0));

        self.write_command(0x45); // set RAM y address start/end
        self.write_data(get_byte!(y_start, 0));
        self.write_data(get_byte!(y_start, 1));
        self.write_data(get_byte!(y_end, 0));
        self.write_data(get_byte!(y_end, 1));

        self.write_command(0x4E); // set RAM x address count to x_start
        self.write_data(get_byte!(x_start, 0));
        self.write_command(0x4F); // set RAM y address count to y_start
        self.write_data(get_byte!(y_start, 0));
        self.write_data(get_byte!(y_start, 1));

        self.write_command(0x24); // Write Black and White image to RAM
        for b in data.iter().flatten() {
            self.write_data(*b);
        }

        self.update(false);
    }

    fn write_command(&mut self, cmd: u8) {
        // DC low for command
        self.dc.set_low().unwrap();
        self.cs.set_low().unwrap();
        self.spi.write(&[cmd]).unwrap();
        self.cs.set_high().unwrap();
    }

    fn write_data(&mut self, data: u8) {
        // DC high for data
        self.dc.set_high().unwrap();
        self.cs.set_low().unwrap();
        self.spi.write(&[data]).unwrap();
        self.cs.set_high().unwrap();
    }

    fn update(&mut self, complete: bool) {
        self.write_command(0x22);
        self.write_data(if complete { 0xF7 } else { 0xFF });
        self.write_command(0x20);
        // Wait for BUSY to go low (Arduino code waits until BUSY==0)
        loop {
            if self.busy.is_low().unwrap_or(false) {
                break;
            }
        }
    }
}
