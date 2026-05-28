//! Basic driver implementation for display: https://www.waveshare.com/wiki/1.5inch_OLED_Module

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::{SpiBus, SpiDevice};

const WIDTH: u8 = 128;
const HEIGHT: u8 = 128;

pub struct Ssd1327<SPI: SpiBus, PIN: OutputPin> {
    spi: SPI,
    dc: PIN,
    // delay: DelayNs,
}

// Public functions
impl<SPI: SpiBus, PIN: OutputPin> Ssd1327<SPI, PIN> {
    pub fn new(spi: SPI, dc: PIN) -> Self {
        Self { spi, dc }
    }

    pub fn init(&mut self) {
        self.write_reg(0xFD);
        self.write_data(0x12);
        self.write_reg(0xFD);
        self.write_data(0xB1);

        self.write_reg(0xAE);
        self.write_reg(0xA4);

        self.write_reg(0x15); // Set column address
        self.write_data(0x00); // Column address start
        self.write_data(WIDTH - 1); // Column address end
        self.write_reg(0x75); // Set row address
        self.write_data(0x00); // Row address start
        self.write_data(HEIGHT - 1); // Row address end

        self.write_reg(0xB3);
        self.write_data(0xF1);

        self.write_reg(0xCA);
        self.write_data(0x7F);

        self.write_reg(0xA0);
        self.write_data(0x74);

        self.write_reg(0xA1);
        self.write_data(0x00);

        self.write_reg(0xA2);
        self.write_data(0x00);

        self.write_reg(0xAB);
        self.write_reg(0x01);

        self.write_reg(0xB4);
        self.write_data(0xA0);
        self.write_data(0xB5);
        self.write_data(0x55);

        self.write_reg(0xC1);
        self.write_data(0xC8);
        self.write_data(0x80);
        self.write_data(0xC0);

        self.write_reg(0xC7);
        self.write_data(0x0F);

        self.write_reg(0xB1);
        self.write_data(0x32);

        self.write_reg(0xB2);
        self.write_data(0xA4);
        self.write_data(0x00);
        self.write_data(0x00);

        self.write_reg(0xBB);
        self.write_data(0x17);

        self.write_reg(0xB6);
        self.write_data(0x01);

        self.write_reg(0xBE);
        self.write_data(0x05);

        self.write_reg(0xA6);
    }

    pub fn clear(&mut self) {
        self.write_reg(0x15);
        self.write_data(0);
        self.write_data(127);
        self.write_reg(0x75);
        self.write_data(0);
        self.write_data(127);

        self.write_reg(0x5C);

        for _ in 0..(WIDTH as u32 * HEIGHT as u32 * 2) {
            self.write_data(0x00);
        }

        // for (uint16_t i = 0; i < displayWidth * displayHeight * 2; i++) {
        //     self.write_data(0x0000);
        // }
    }
}

// Private functions
impl<SPI: SpiBus, PIN: OutputPin> Ssd1327<SPI, PIN> {
    fn write_reg(&mut self, reg: u8) {
        self.dc.set_low().unwrap();
        self.spi.write(&[reg]).unwrap();
    }

    fn write_data(&mut self, data: u8) {
        self.dc.set_high().unwrap();
        self.spi.write(&[data]).unwrap();
    }
}
