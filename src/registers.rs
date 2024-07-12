#![allow(dead_code, unused)]

use core::fmt::Display;

const DEFAULT_ADDRESS: u8 = 0x20; // Default address
                                  ////// STATES ////////
#[derive(Debug, Clone)]
pub struct Configuring;

#[derive(Debug, Clone)]
pub struct OutputReady;

#[derive(Debug, Clone)]
pub struct InputConfiguring;

#[derive(Debug, Clone)]
pub struct InputReady;

pub enum Register {
    Iodir = 0x00,
    Ipol = 0x02,
    Gpinten = 0x04,
    Defval = 0x06,
    Intcon = 0x08,
    Iocon = 0x0A,
    Gppu = 0x0C,
    Intf = 0x0E,
    Intcap = 0x10,
    Gpio = 0x12,
    Olat = 0x14,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PinMask {
    Pin0 = 0x01,
    Pin1 = 0x02,
    Pin2 = 0x04,
    Pin3 = 0x08,
    Pin4 = 0x10,
    Pin5 = 0x20,
    Pin6 = 0x40,
    Pin7 = 0x80,
    Invalid = 0x00,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PinNumber {
    Pin0,
    Pin1,
    Pin2,
    Pin3,
    Pin4,
    Pin5,
    Pin6,
    Pin7,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MyPort {
    Porta = 0x00,
    Portb = 0x01,
}

/// Enum used for mcp23017 addressing based on pin connection
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SlaveAddressing {
    Low,
    High,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PinSet {
    Low = 0,
    High = 1,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Output = 0,
    Input = 1,
}

///Valid error codes
#[derive(Debug, PartialEq)]
pub enum Error {
    CommunicationErr,
    InvalidParameter,
    InvalidDie,
    InvalidManufacturer,
    MissingAddress,
    MissingI2C,
    PinIsNotInput,
    InvalidInterruptSetting,
}

pub enum InterruptOn {
    PinChange = 0,
    ChangeFromRegister = 1,
}

pub enum InterruptMirror {
    MirrorOn = 0b01000000,
    MirrorOff = 0b10111111,
}

impl From<u8> for PinMask {
    fn from(value: u8) -> Self {
        match value {
            0x01 => PinMask::Pin0,
            0x02 => PinMask::Pin1,
            0x04 => PinMask::Pin2,
            0x08 => PinMask::Pin3,
            0x10 => PinMask::Pin4,
            0x20 => PinMask::Pin5,
            0x40 => PinMask::Pin6,
            0x80 => PinMask::Pin7,
            _ => PinMask::Invalid,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidDie => write!(f, "Invalid Die Number"),
            Error::CommunicationErr => write!(f, "Not found on address"),
            Error::InvalidManufacturer => write!(f, "Invalid Manufacturer"),
            Error::InvalidParameter => write!(f, "Invalid Parameter"),
            Error::MissingAddress => write!(f, "Missing Device Address"),
            Error::MissingI2C => write!(f, "Missing I2C Bus"),
            Error::PinIsNotInput => write!(f, "Pin is not Input"),
            Error::InvalidInterruptSetting => write!(f, "Invalid Interrupt Setting"),
        }
    }
}

impl Display for SlaveAddressing {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SlaveAddressing::High => write!(f, "High"),
            SlaveAddressing::Low => write!(f, "Low"),
        }
    }
}

/**
 * Returns communication error
 */
pub fn i2c_comm_error<E>(_: E) -> Error {
    Error::CommunicationErr
}

pub fn pin_number_to_mask(pin: PinNumber) -> PinMask {
    match pin {
        PinNumber::Pin0 => PinMask::Pin0,
        PinNumber::Pin1 => PinMask::Pin1,
        PinNumber::Pin2 => PinMask::Pin2,
        PinNumber::Pin3 => PinMask::Pin3,
        PinNumber::Pin4 => PinMask::Pin4,
        PinNumber::Pin5 => PinMask::Pin5,
        PinNumber::Pin6 => PinMask::Pin6,
        PinNumber::Pin7 => PinMask::Pin7,
    }
}

#[allow(dead_code)]
pub fn pin_mask_to_number(pin: PinMask) -> Option<PinNumber> {
    match pin {
        PinMask::Pin0 => Some(PinNumber::Pin0),
        PinMask::Pin1 => Some(PinNumber::Pin1),
        PinMask::Pin2 => Some(PinNumber::Pin2),
        PinMask::Pin3 => Some(PinNumber::Pin3),
        PinMask::Pin4 => Some(PinNumber::Pin4),
        PinMask::Pin5 => Some(PinNumber::Pin5),
        PinMask::Pin6 => Some(PinNumber::Pin6),
        PinMask::Pin7 => Some(PinNumber::Pin7),
        PinMask::Invalid => None,
    }
}

pub fn bit_set(byte: u8, pin: PinNumber) -> u8 {
    byte | (pin_number_to_mask(pin) as u8)
}

pub fn bit_clear(byte: u8, pin: PinNumber) -> u8 {
    byte & !(pin_number_to_mask(pin) as u8)
}

pub fn bit_read(byte: u8, pin: PinNumber) -> u8 {
    (byte & (pin_number_to_mask(pin) as u8)) >> (pin as u8)
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;
    use crate::bit_read;
    extern crate embedded_hal_mock;
    extern crate std;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

    #[test]
    fn test_bit_set() {
        let mut value = 0b00000000;

        value = bit_set(value, PinNumber::Pin7);

        println!("value 0b{:08b}", value);
        assert_eq!(0b10000000, value);
    }

    #[test]
    fn test_bit_clear() {
        let mut value = 0b11111111;

        value = bit_clear(value, PinNumber::Pin7);

        println!("value 0b{:08b}", value);
        assert_eq!(0b01111111, value);
    }

    #[test]
    fn test_bit_read() {
        let mut value = 0b10000000;

        value = bit_read(value, PinNumber::Pin7);

        println!("value 0b{:08b}", value);
        assert_eq!(0b00000001, value);
    }
}
