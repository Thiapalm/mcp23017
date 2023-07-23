#![no_std]

// Imports
use byteorder::{BigEndian, ByteOrder};
use core::fmt::Display;
use embedded_hal::blocking::i2c::{Write, WriteRead};

#[allow(dead_code)]
enum Register {
    Configuration = 0x00,
    ShuntVoltage = 0x01,
    BusVoltage = 0x02,
    Power = 0x03,
    Current = 0x04,
    Calibration = 0x05,
    MaskEnable = 0x06,
    Alert = 0x07,
    Manufacturer = 0xFE,
    DieId = 0xFF,
}

/// Enum used for mcp23017 addressing based on pin connection
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SlaveAddressing {
    Low,
    High,
}

impl Display for SlaveAddressing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SlaveAddressing::High => write!(f, "High"),
            SlaveAddressing::Low => write!(f, "Low"),
        }
    }
}

///Valid error codes
#[derive(Debug, PartialEq)]
pub enum Error {
    CommunicationErr,
    InvalidParameter,
    InvalidDie,
    InvalidManufacturer,
    MissingAddress,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidDie => write!(f, "Invalid Die Number"),
            Error::CommunicationErr => write!(f, "Not found on address"),
            Error::InvalidManufacturer => write!(f, "Invalid Manufacturer"),
            Error::InvalidParameter => write!(f, "Invalid Parameter"),
            Error::MissingAddress => write!(f, "Missing Device Address"),
        }
    }
}

/**
 * Returns communication error
 */
fn i2c_comm_error<E>(_: E) -> Error {
    Error::CommunicationErr
}

/**
 * Function that converts physical pin address connection to respective hexadecimal value
 */
pub fn convert_slave_address(a0: SlaveAddressing, a1: SlaveAddressing, a2: SlaveAddressing) -> u8 {
    match (a0, a1, a2) {
        (SlaveAddressing::Low, SlaveAddressing::Low, SlaveAddressing::Low) => 0x20,
        (SlaveAddressing::Low, SlaveAddressing::Low, SlaveAddressing::High) => 0x21,
        (SlaveAddressing::Low, SlaveAddressing::High, SlaveAddressing::Low) => 0x22,
        (SlaveAddressing::Low, SlaveAddressing::High, SlaveAddressing::High) => 0x23,
        (SlaveAddressing::High, SlaveAddressing::Low, SlaveAddressing::Low) => 0x24,
        (SlaveAddressing::High, SlaveAddressing::Low, SlaveAddressing::High) => 0x25,
        (SlaveAddressing::High, SlaveAddressing::High, SlaveAddressing::Low) => 0x26,
        (SlaveAddressing::High, SlaveAddressing::High, SlaveAddressing::High) => 0x27,
    }
}

trait Operation<I2C, E> {
    fn write(&mut self, i2c: &mut I2C, register: Register, buf: &mut [u8; 2]) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>;
    fn read(&mut self, i2c: &mut I2C, register: Register) -> Result<[u8; 2], Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>;
}

//todo 1. Faça funcionar
//todo 2. generalize
//todo 3. imponha restrições

pub struct Mcp23017 {
    address: Option<u8>,
}

impl Default for Mcp23017 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mcp23017 {
    pub fn new() -> Self {
        Mcp23017 { address: None }
    }

    pub fn set_address(&mut self, address: u8) -> &mut Self {
        self.address = Some(address);
        self
    }

    pub fn find_me<I2C, E>(&mut self, i2c: &mut I2C) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let result = self.read(i2c, Register::Configuration);
        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::CommunicationErr),
        }
    }
}

impl<I2C, E> Operation<I2C, E> for Mcp23017
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    fn write(&mut self, i2c: &mut I2C, register: Register, buf: &mut [u8; 2]) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let address = self.address.unwrap();
        let _ = i2c.write(address, &[register as u8, buf[1], buf[0]]);
        Ok(())
    }

    fn read(&mut self, i2c: &mut I2C, register: Register) -> Result<[u8; 2], Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let mut rx_buffer: [u8; 2] = [0; 2];
        let address = self.address.unwrap();

        i2c.write_read(address, &[register as u8], &mut rx_buffer)
            .map_err(i2c_comm_error)?;
        Ok(rx_buffer)
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
