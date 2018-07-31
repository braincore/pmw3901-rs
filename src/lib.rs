//! A library for the PMW3901 optical flow sensor.

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};
extern crate spidev;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_3};
use std::io;
use std::thread;
use std::time;

/// Motion output of the sensor. 
#[derive(Debug)]
pub struct Pmw3901Sample {
    /// Unit is pixel velocity.
    pub x: i16,
    /// Unit is pixel velocity.
    pub y: i16,
}

/// Optical flow sensor.
pub struct Pmw3901 {
    spi_dev: Spidev,
    pub debug: bool,
}

impl Pmw3901 {

    // Initializes the SPI connection but does not use it.
    pub fn new(bus: u8, chip_select: u8) -> io::Result<Pmw3901> {
        let mut spi_dev = Spidev::open(
            format!("/dev/spidev{}.{}", bus, chip_select))?;
        let options = SpidevOptions::new()
             .bits_per_word(8)
             .max_speed_hz(2_000_000)
             .lsb_first(false)
             .mode(SPI_MODE_3)
             .build();
        spi_dev.configure(&options)?;
        Ok(Pmw3901 {
            spi_dev,
            debug: false,
        })
    }

    /// Read a value from a register.
    pub fn read_register(&mut self, addr: u8) -> io::Result<u8> {
        let tx_buf = [addr, 0];
        let mut rx_buf = [0; 2];
        {
            let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
            self.spi_dev.transfer(&mut transfer)?;
        }
        if rx_buf[0] != 0xff {
            panic!("Unexpected first byte in read response: {}", rx_buf[0]);
        }
        Ok(rx_buf[1])
    }

    /// Write a value to a register.
    pub fn write_register(&mut self, addr: u8, val: u8) -> io::Result<u8> {
        if addr & 0x80 > 0 {
            panic!("Write bit already set on addr: {}", addr);
        }
        let tx_buf = [addr | 0x80, val];
        let mut rx_buf = [0; 2];
        {
            let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
            self.spi_dev.transfer(&mut transfer)?;
        }
        if rx_buf[0] != 0xff {
            panic!("Unexpected first byte in read response: {}", rx_buf[0]);
        }
        if rx_buf[1] != 0xff {
            panic!("Unexpected second byte in read response: {}", rx_buf[0]);
        }
        Ok(rx_buf[1])
    }

    /// Helper for batch reading from multiple registers.
    pub fn read_registers(&mut self, addrs: &[u8]) -> io::Result<Vec<u8>> {
        let mut bufs = Vec::new();
        for addr in addrs {
            bufs.push(([*addr, 0], [0, 0]));
        } 
        {
            let mut transfers = Vec::new();
            for buf in bufs.iter_mut() {
                let transfer = SpidevTransfer::read_write(&buf.0, &mut buf.1);
                transfers.push(transfer);
            }
            self.spi_dev.transfer_multiple(&mut transfers)?;
        }
        let mut res = Vec::new();
        for buf in bufs {
            if buf.1[0] != 0xff {
                panic!("Unexpected first byte in read response: {}", buf.1[0]);
            }
            res.push(buf.1[1]);
        }
        Ok(res)
    }

    /// Helper for batch writing to multiple registers.
    pub fn write_registers(&mut self, addrs_and_values: &[(u8, u8)]) -> io::Result<()> {
        let mut bufs = Vec::new();
        for &(addr, val) in addrs_and_values {
            bufs.push(([addr | 0x80, val], [0, 0]));
        } 
        {
            let mut transfers = Vec::new();
            for buf in bufs.iter_mut() {
                let transfer = SpidevTransfer::read_write(&buf.0, &mut buf.1);
                transfers.push(transfer);
            }
            self.spi_dev.transfer_multiple(&mut transfers)?;
        }
        for buf in bufs {
            if buf.1[0] != 0xff {
                panic!("Unexpected first byte in write response: {}", buf.1[0]);
            }
            if buf.1[0] != 0xff {
                panic!("Unexpected second byte in write response: {}", buf.1[0]);
            }
        }
        Ok(())
    }

    /// Initializes the device.
    /// * Validates known registers (Product ID).
    /// * Initializes device configuration.
    pub fn init(&mut self) -> io::Result<()> {
        // Power on reset
        self.write_register(0x3a, 0x5a)?;

        // Verify product id
        let product_id = self.read_register(0x00)?;
        if product_id != 0x49 {
            panic!("Unexpected product id: {} (expected 0x49)", product_id);
        }
        if self.debug {
            println!("Product ID: {:?}", product_id);
        }

        // Verify inverse product id
        let inverse_product_id = self.read_register(0x5f)?;
        if inverse_product_id != 0xb6 {
            panic!("Unexpected inverse product id: {} (expected 0xb6)", product_id);
        }
        if self.debug {
            println!("Inverse Product ID: {:?}", inverse_product_id);
        }

        self.write_init_registers()?;

        Ok(())
    }

    /// This is black magic taken from the BitCraze source.
    fn write_init_registers(&mut self) -> io::Result<()> {
        self.write_registers(&[
            (0x7F, 0x00),
            (0x61, 0xAD),
            (0x7F, 0x03),
            (0x40, 0x00),
            (0x7F, 0x05),
            (0x41, 0xB3),
            (0x43, 0xF1),
            (0x45, 0x14),
            (0x5B, 0x32),
            (0x5F, 0x34),
            (0x7B, 0x08),
            (0x7F, 0x06),
            (0x44, 0x1B),
            (0x40, 0xBF),
            (0x4E, 0x3F),
            (0x7F, 0x08),
            (0x65, 0x20),
            (0x6A, 0x18),
            (0x7F, 0x09),
            (0x4F, 0xAF),
            (0x5F, 0x40),
            (0x48, 0x80),
            (0x49, 0x80),
            (0x57, 0x77),
            (0x60, 0x78),
            (0x61, 0x78),
            (0x62, 0x08),
            (0x63, 0x50),
            (0x7F, 0x0A),
            (0x45, 0x60),
            (0x7F, 0x00),
            (0x4D, 0x11),
            (0x55, 0x80),
            (0x74, 0x1F),
            (0x75, 0x1F),
            (0x4A, 0x78),
            (0x4B, 0x78),
            (0x44, 0x08),
            (0x45, 0x50),
            (0x64, 0xFF),
            (0x65, 0x1F),
            (0x7F, 0x14),
            (0x65, 0x60),
            (0x66, 0x08),
            (0x63, 0x78),
            (0x7F, 0x15),
            (0x48, 0x58),
            (0x7F, 0x07),
            (0x41, 0x0D),
            (0x43, 0x14),
            (0x4B, 0x0E),
            (0x45, 0x0F),
            (0x44, 0x42),
            (0x4C, 0x80),
            (0x7F, 0x10),
            (0x5B, 0x02),
            (0x7F, 0x07),
            (0x40, 0x41),
            (0x70, 0x00),
        ])?;

        thread::sleep(time::Duration::from_millis(100));

        self.write_registers(&[
            (0x32, 0x44),
            (0x7F, 0x07),
            (0x40, 0x40),
            (0x7F, 0x06),
            (0x62, 0xf0),
            (0x63, 0x00),
            (0x7F, 0x0D),
            (0x48, 0xC0),
            (0x6F, 0xd5),
            (0x7F, 0x00),
            (0x5B, 0xa0),
            (0x4E, 0xA8),
            (0x5A, 0x50),
            (0x40, 0x80),
        ])?;

        Ok(())
    }

    /// Reads the x/y delta registers.
    pub fn read_sample(&mut self) -> io::Result<Pmw3901Sample> {
        //self.read_register(0x02)?;
        //Ok(Pmw3901Sample {
        //    x: LittleEndian::read_i16(&[self.read_register(0x03)?, self.read_register(0x04)?]),
        //    y: LittleEndian::read_i16(&[self.read_register(0x05)?, self.read_register(0x06)?]),
        //})
        let res = self.read_registers(&[0x02, 0x03, 0x04, 0x05, 0x06])?;
        Ok(Pmw3901Sample {
            x: LittleEndian::read_i16(&res[1 .. 3]),
            y: LittleEndian::read_i16(&res[3 .. 5]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Pmw3901};
    use std::env;

    fn get_spi_bus() -> u8 {
        match env::var("PMW3901_SPI_BUS") {
            Ok(bus_string) => {
                bus_string.parse().expect(
                    "Could not convert PMW3901_SPI_BUS env var to u8.")
            },
            Err(_) => {
                panic!("PMW3901_SPI_BUS must be specified.");
            },
        }
    }

    fn get_spi_cs() -> u8 {
        match env::var("PMW3901_SPI_CS") {
            Ok(addr_string) => {
                addr_string.parse().expect(
                    "Could not convert PMW3901_SPI_CS env var to u8.")
            },
            Err(_) => {
                panic!("PMW3901_SPI_CS must be specified.");
            },
        }
    }

    #[test]
    fn basic() {
        let mut pmw3901 = Pmw3901::new(
            get_spi_bus(), get_spi_cs()).unwrap();
        pmw3901.init().unwrap();
        pmw3901.read_sample().unwrap();
    }
}
