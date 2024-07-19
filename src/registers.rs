#![allow(dead_code, unused)]

use crate::prelude::*;
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

/**
 * Function implements the From trait into PinMask enum
 */
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

/**
 * Function implements the Display trait into Error enum
 */
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

/**
 * Function implements the Display trait into Register enum
 */
impl Display for Register {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Register::Iodir => write!(f, "Iodir (0x00)"),
            Register::Ipol => write!(f, "Ipol (0x02)"),
            Register::Gpinten => write!(f, "Gpinten (0x04)"),
            Register::Defval => write!(f, "Defval (0x06)"),
            Register::Intcon => write!(f, "Intcon (0x08)"),
            Register::Iocon => write!(f, "Iocon (0x0A)"),
            Register::Gppu => write!(f, "Gppu (0x0C)"),
            Register::Intf => write!(f, "Intf (0x0E)"),
            Register::Intcap => write!(f, "Intcap (0x10)"),
            Register::Gpio => write!(f, "Gpio (0x12)"),
            Register::Olat => write!(f, "Olat (0x14)"),
        }
    }
}

/**
 * Function implements the Display trait into Myport enum
 */
impl Display for MyPort {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MyPort::Porta => write!(f, "Porta (0x00)"),
            MyPort::Portb => write!(f, "Portb (0x01)"),
        }
    }
}

/**
 * Function implements the Display trait into SlaveAddressing enum
 */
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

/**
 * Function used to convert a pin number to a pin mask
 */
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

/**
 * This function converts a pin mask to a pin number
 */
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

/**
 * This function is used to set a given bit. It must receive the byte to be changed
 * and the pin number to set
 */
pub fn bit_set(byte: u8, pin: PinNumber) -> u8 {
    byte | (pin_number_to_mask(pin) as u8)
}

/**
 * This function is used to clear a given bit. It must receive the byte to be changed
 * and the pin number to be cleared
 */
pub fn bit_clear(byte: u8, pin: PinNumber) -> u8 {
    byte & !(pin_number_to_mask(pin) as u8)
}

/**
 * This function reads a given bit from a byte. It must receive the byte and
 * the pin number to be read
 */
pub fn bit_read(byte: u8, pin: PinNumber) -> u8 {
    (byte & (pin_number_to_mask(pin) as u8)) >> (pin as u8)
}

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;
    use crate::registers::bit_read;
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
