use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus;

pub const EPD_WIDTH: usize = 250;
pub const EPD_HEIGHT: usize = 122;
pub const EPD_BUFFER_SIZE: usize = EPD_WIDTH * EPD_HEIGHT / 8;

pub struct PaperDisplay<SPI, CS, DC, RST, BUSY> {
    pub spi: SPI,
    pub cs: CS,
    pub dc: DC,
    pub rst: RST,
    pub busy: BUSY,
    pub buffer: [u8; EPD_BUFFER_SIZE],
}

impl<SPI, CS, DC, RST, BUSY> PaperDisplay<SPI, CS, DC, RST, BUSY>
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
    BUSY: InputPin,
{
    pub fn reset(&mut self, delay: &mut impl embedded_hal::delay::DelayNs) {
        self.rst.set_low().ok();
        delay.delay_ms(10);
        self.rst.set_high().ok();
        delay.delay_ms(10);
        while !self.busy.is_low().unwrap_or(false) {}
    }

    pub fn init(&mut self, delay: &mut impl embedded_hal::delay::DelayNs) {
        self.reset(delay);
        self.write_command(0x12); // SWRESET
        while !self.busy.is_low().unwrap_or(false) {}
        self.fill_screen(0xFF); // White
    }

    pub fn fill_screen(&mut self, color: u8) {
        for b in self.buffer.iter_mut() {
            *b = color;
        }
    }

    pub fn set_ram_area(
        &mut self,
        x_start: u8,
        x_end: u8,
        y_start: u8,
        y_start1: u8,
        y_end: u8,
        y_end1: u8,
    ) {
        self.write_command(0x44);
        self.write_data(&[x_start, x_end]);
        self.write_command(0x45);
        self.write_data(&[y_start, y_start1, y_end, y_end1]);
    }

    pub fn set_ram_pointer(&mut self, addr_x: u8, addr_y: u8, addr_y1: u8) {
        self.write_command(0x4E);
        self.write_data(&[addr_x]);
        self.write_command(0x4F);
        self.write_data(&[addr_y, addr_y1]);
    }

    pub fn write_full_screen(&mut self) {
        self.write_command(0x24);
        let buffer = self.buffer.clone();
        for &b in &buffer {
            self.write_data(&[b]);
        }
        self.update_full();
    }

    pub fn write_window(&mut self, x: u16, y: u16, w: u16, h: u16) {
        let xe = (x + w).min(EPD_WIDTH as u16) - 1;
        let ye = (y + h).min(EPD_HEIGHT as u16) - 1;
        let xs_d8 = x / 8;
        let xe_d8 = xe / 8;
        self.set_ram_area(
            xs_d8 as u8,
            xe_d8 as u8,
            (y % 256) as u8,
            (y / 256) as u8,
            (ye % 256) as u8,
            (ye / 256) as u8,
        );
        self.set_ram_pointer(xs_d8 as u8, (y % 256) as u8, (y / 256) as u8);
        self.write_command(0x24);
        for y1 in y..=ye {
            for x1 in xs_d8..=xe_d8 {
                let idx = y1 as usize * (EPD_WIDTH / 8) + x1 as usize;
                let data = if idx < EPD_BUFFER_SIZE {
                    self.buffer[idx]
                } else {
                    0xFF
                };
                self.write_data(&[data]);
            }
        }
        self.update_part();
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, color: bool) {
        if x >= EPD_WIDTH as u16 || y >= EPD_HEIGHT as u16 {
            return;
        }
        let idx = (y as usize) * (EPD_WIDTH / 8) + (x as usize) / 8;
        let bit = 7 - (x % 8);
        if color {
            self.buffer[idx] &= !(1 << bit);
        } else {
            self.buffer[idx] |= 1 << bit;
        }
    }

    fn write_command(&mut self, cmd: u8) {
        self.dc.set_low().ok();
        self.cs.set_low().ok();
        self.spi.write(&[cmd]).ok();
        self.cs.set_high().ok();
    }

    fn write_data(&mut self, data: &[u8]) {
        self.dc.set_high().ok();
        self.cs.set_low().ok();
        self.spi.write(data).ok();
        self.cs.set_high().ok();
    }

    fn update_full(&mut self) {
        self.write_command(0x22);
        self.write_data(&[0xF7]);
        self.write_command(0x20);
        self.wait_busy();
    }

    fn update_part(&mut self) {
        self.write_command(0x22);
        self.write_data(&[0xFF]);
        self.write_command(0x20);
        self.wait_busy();
    }

    fn wait_busy(&mut self) {
        for _ in 0..400 {
            if self.busy.is_low().unwrap_or(false) {
                break;
            }
        }
    }
}
