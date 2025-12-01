use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::spi::SpiBus;

pub const EPD_WIDTH: usize = 250;
pub const EPD_HEIGHT: usize = 122;
pub const EPD_BUFFER_SIZE: usize = EPD_WIDTH * EPD_HEIGHT / 8;

pub const EPD_LUT_DEFAULT_FULL: [u8; 100] = [
    0xA0, 0x90, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x90, 0xA0, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0xA0, 0x90, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x90,
    0xA0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x0F, 0x0F, 0x00, 0x00, 0x00, 0x0F, 0x0F, 0x00, 0x00, 0x03, 0x0F, 0x0F, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

pub const EPD_LUT_DEFAULT_PART: [u8; 100] = [
    0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

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
        self.wait_busy();
    }

    pub fn fill_screen(&mut self, color: u8) {
        for b in self.buffer.iter_mut() {
            *b = color;
        }
    }

    pub fn power_on(&mut self) {
        self.write_command(0x22);
        self.write_data(&[0xc0]);
        self.write_command(0x20);
        self.wait_busy();
    }

    pub fn power_off(&mut self) {
        self.write_command(0x22);
        self.write_data(&[0xc3]);
        self.write_command(0x20);
        self.wait_busy();
    }

    pub fn update(&mut self) {
        self.init_full(0x03);
        self.write_command(0x24);
        for y in 0..EPD_HEIGHT {
            for x in 0..(EPD_WIDTH / 8) {
                let idx = y * (EPD_WIDTH / 8) + x;
                let data = if idx < EPD_BUFFER_SIZE {
                    self.buffer[idx]
                } else {
                    0xFF
                };
                self.write_data(&[data]);
            }
        }
        self.update_full();
    }

    pub fn update_window(&mut self, x: u16, y: u16, w: u16, h: u16) {
        let xe = (x + w).min(EPD_WIDTH as u16) - 1;
        let ye = (y + h).min(EPD_HEIGHT as u16) - 1;
        let xs_d8 = x / 8;
        let xe_d8 = xe / 8;

        self.init_part(0x3);

        self.set_ram_area(
            xs_d8 as u8,
            xe_d8 as u8,
            (y % 256) as u8,
            (y / 256) as u8,
            (ye % 256) as u8,
            (ye / 256) as u8,
        );
        self.set_ram_pointer(xs_d8 as u8, (y % 256) as u8, (y / 256) as u8);

        self.wait_busy();
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

    /// Display register initialization (port from epd_init_display)
    fn init_display(&mut self, em: u8) {
        // set analog block control
        self.write_command(0x74);
        self.write_data(&[0x54]);
        // set digital block control
        self.write_command(0x7e);
        self.write_data(&[0x3b]);
        // driver output control
        self.write_command(0x01);
        self.write_data(&[0xf9, 0x00, 0x00]);
        // data entry mode
        self.write_command(0x11);
        self.write_data(&[0x01]);
        // set ram area
        self.set_ram_area(0x00, 0x0f, 0xf9, 0x00, 0x00, 0x00);
        // border wave form
        self.write_command(0x3c);
        self.write_data(&[0x03]);
        // vcom voltage
        self.write_command(0x2c);
        self.write_data(&[0x50]);
        // gate driving voltage Control
        self.write_command(0x03);
        self.write_data(&[0x15]);
        // source driving voltage Control
        self.write_command(0x04);
        self.write_data(&[0x41, 0xa8, 0x32]);
        // dummy line
        self.write_command(0x3a);
        self.write_data(&[0x2c]);
        // gate time
        self.write_command(0x3b);
        self.write_data(&[0x0b]);
        // set ram pointer
        self.set_ram_pointer(0x00, 0xf9, 0x00);
        self.set_ram_data_entry_mode(em);
        // Data entry mode and RAM area setup omitted for brevity
    }

    fn init_full(&mut self, em: u8) {
        self.init_display(em);
        self.write_command(0x32);
        self.write_data(&EPD_LUT_DEFAULT_FULL);
        self.power_on();
    }

    fn init_part(&mut self, em: u8) {
        self.init_display(em);
        self.write_command(0x2c);
        self.write_data(&[0x26]);
        self.write_command(0x32);
        self.write_data(&EPD_LUT_DEFAULT_PART);
        self.power_on();
    }

    fn update_full(&mut self) {
        self.write_command(0x22);
        self.write_data(&[0xf7]);
        self.write_command(0x20);
        self.wait_busy();
    }

    fn update_part(&mut self) {
        self.write_command(0x22);
        self.write_data(&[0xff]);
        self.write_command(0x20);
        self.wait_busy();
    }

    /// Set RAM data entry mode (port from epd_set_ram_data_entry_mode)
    pub fn set_ram_data_entry_mode(&mut self, em: u8) {
        let x_pixels_par = EPD_WIDTH as u16 - 1;
        let y_pixels_par = EPD_HEIGHT as u16 - 1;
        let em = em.min(0x03);
        self.write_command(0x11);
        self.write_data(&[em]);
        match em {
            0x00 => {
                // x decrease, y decrease
                self.set_ram_area(
                    (x_pixels_par / 8) as u8,
                    0x00,
                    (y_pixels_par % 256) as u8,
                    (y_pixels_par / 256) as u8,
                    0x00,
                    0x00,
                );
                self.set_ram_pointer(
                    (x_pixels_par / 8) as u8,
                    (y_pixels_par % 256) as u8,
                    (y_pixels_par / 256) as u8,
                );
            }
            0x01 => {
                // x increase, y decrease
                self.set_ram_area(
                    0x00,
                    (x_pixels_par / 8) as u8,
                    (y_pixels_par % 256) as u8,
                    (y_pixels_par / 256) as u8,
                    0x00,
                    0x00,
                );
                self.set_ram_pointer(0x00, (y_pixels_par % 256) as u8, (y_pixels_par / 256) as u8);
            }
            0x02 => {
                // x decrease, y increase
                self.set_ram_area(
                    (x_pixels_par / 8) as u8,
                    0x00,
                    0x00,
                    0x00,
                    (y_pixels_par % 256) as u8,
                    (y_pixels_par / 256) as u8,
                );
                self.set_ram_pointer((x_pixels_par / 8) as u8, 0x00, 0x00);
            }
            0x03 => {
                // x increase, y increase
                self.set_ram_area(
                    0x00,
                    (x_pixels_par / 8) as u8,
                    0x00,
                    0x00,
                    (y_pixels_par % 256) as u8,
                    (y_pixels_par / 256) as u8,
                );
                self.set_ram_pointer(0x00, 0x00, 0x00);
            }
            _ => {}
        }
    }

    /// Set RAM area (port from epd_set_ram_area)
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

    /// Set RAM pointer (port from epd_set_ram_pointer)
    pub fn set_ram_pointer(&mut self, addr_x: u8, addr_y: u8, addr_y1: u8) {
        self.write_command(0x4E);
        self.write_data(&[addr_x]);
        self.write_command(0x4F);
        self.write_data(&[addr_y, addr_y1]);
    }

    pub fn init(&mut self, delay: &mut impl embedded_hal::delay::DelayNs) {
        self.reset(delay);
        self.write_command(0x12); // SWRESET
        self.wait_busy();
        self.fill_screen(0xFF); // White
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

    fn wait_busy(&mut self) {
        while !self.busy.is_low().unwrap_or(false) {}
    }
}
