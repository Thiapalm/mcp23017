#![no_std]

/////// Imports

//use byteorder::{BigEndian, ByteOrder};
use byteorder::{ByteOrder, LittleEndian};
use core::fmt::Display;
use embedded_hal::i2c;
//use rtt_target::rprintln;

//const DEFAULT_ADDRESS: u8 = 0x20; // Default address
#[allow(dead_code)]
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
enum PinMask {
    Pin0 = 0x01,
    Pin1 = 0x02,
    Pin2 = 0x04,
    Pin3 = 0x08,
    Pin4 = 0x10,
    Pin5 = 0x20,
    Pin6 = 0x40,
    Pin7 = 0x80,
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

#[derive(Debug)]
pub struct Configuring;
#[derive(Debug)]
pub struct ChipMode;
#[derive(Debug)]
pub struct PortMode;
#[derive(Debug)]
pub struct PinMode;

pub struct Mcp23017<I2C, State = Configuring> {
    i2c: I2C,
    address: u8,
    state: core::marker::PhantomData<State>,
}

pub enum InterruptOn {
    PinChange = 0,
    ChangeFromRegister = 1,
}

pub enum InterruptMirror {
    MirrorOn = 0b01000000,
    MirrorOff = 0b10111111,
}
/////// Support functions

fn pin_number_to_mask(pin: PinNumber) -> PinMask {
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

fn pin_mask_to_number(pin: PinMask) -> PinNumber {
    match pin {
        PinMask::Pin0 => PinNumber::Pin0,
        PinMask::Pin1 => PinNumber::Pin1,
        PinMask::Pin2 => PinNumber::Pin2,
        PinMask::Pin3 => PinNumber::Pin3,
        PinMask::Pin4 => PinNumber::Pin4,
        PinMask::Pin5 => PinNumber::Pin5,
        PinMask::Pin6 => PinNumber::Pin6,
        PinMask::Pin7 => PinNumber::Pin7,
    }
}

fn bit_set(byte: u8, pin: PinNumber) -> u8 {
    byte | (pin_number_to_mask(pin) as u8)
}

fn bit_clear(byte: u8, pin: PinNumber) -> u8 {
    byte & !(pin_number_to_mask(pin) as u8)
}

fn bit_read(byte: u8, pin: PinNumber) -> u8 {
    (byte & (pin_number_to_mask(pin) as u8)) >> (pin as u8)
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

/////// Traits
pub trait Configuration {
    fn write_config(&mut self, register: Register, port: MyPort, value: u8) -> Result<(), Error>;
    fn read_config(&mut self, register: Register, port: MyPort) -> Result<u8, Error>;
    fn set_pin_dir(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        direction: Direction,
    ) -> Result<(), Error>;
    fn set_pull(&mut self, port: MyPort, pin: PinNumber, pull: PinSet) -> Result<(), Error>; //set GPPU (1 up, 0 disabled) return error if pin is output
}

pub trait Interrupts {
    fn find_interrupted_pin(&mut self, port: MyPort) -> PinNumber; //Read INTF register
    fn set_mirror(&mut self, mirror: InterruptMirror) -> Result<(), Error>; //Set IOCON.MIRROR Return error only on comm failure
    fn set_interrupt_on(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        interrupt_on: InterruptOn,
    ) -> Result<(), Error>; //set INTCON register
    fn set_interrupt_compare(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        value: PinSet,
    ) -> Result<(), Error>; //det DEFVAL register, only valid if INTCON is set to 1
    fn enable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error>; //Set GPINTEN, only valid if DEFVAL and INTCON already configured
    fn disable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error>;
}

pub trait MyOutput {
    fn write(&mut self, value: u16) -> Result<(), Error>;
    fn write_port(&mut self, port: MyPort, value: u8) -> Result<(), Error>;
    fn write_pin(&mut self, port: MyPort, pin: PinNumber, value: PinSet) -> Result<(), Error>;
}

pub trait MyInput {
    fn read(&mut self) -> Result<u16, Error>;
    fn read_port(&mut self, port: MyPort) -> Result<u8, Error>;
    fn read_pin(&mut self, port: MyPort, pin: PinNumber) -> Result<u8, Error>;
}

/////// Impls

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
            _ => PinMask::Pin0,
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

// impl Default for Mcp23017<I2C, State> {
//     fn default() -> Self {

//     }

// }

impl<I2C, E, State> Mcp23017<I2C, State>
where
    I2C: i2c::I2c<Error = E>,
{
    pub fn new(i2c: I2C, address: u8) -> Self {
        Mcp23017 {
            i2c,
            address,
            state: Default::default(),
        }
    }
}

impl<I2C, E> Configuration for Mcp23017<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    fn read_config(&mut self, register: Register, port: MyPort) -> Result<u8, Error> {
        let register_address = register as u8 | port as u8;
        let mut rx_buffer: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .map_err(i2c_comm_error)?;
        Ok(rx_buffer[0])
    }

    fn write_config(&mut self, register: Register, port: MyPort, value: u8) -> Result<(), Error> {
        let register_address = register as u8 | port as u8;
        self.i2c
            .write(self.address, &[register_address, value])
            .map_err(i2c_comm_error)?;
        Ok(())
    }

    fn set_pin_dir(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        direction: Direction,
    ) -> Result<(), Error> {
        let mut value = self.read_config(Register::Iodir, port).unwrap();

        value = match direction {
            Direction::Input => bit_set(value, pin),
            Direction::Output => bit_clear(value, pin),
        };

        self.write_config(Register::Iodir, port, value)
    }

    fn set_pull(&mut self, port: MyPort, pin: PinNumber, pull: PinSet) -> Result<(), Error> {
        let mut pin_is_output = self.read_config(Register::Iodir, port)?;
        pin_is_output = bit_read(pin_is_output, pin);

        if pin_is_output == Direction::Output as u8 {
            return Err(Error::PinIsNotInput);
        }

        let result = self.read_config(Register::Gppu, port)?;

        let result = match pull {
            PinSet::High => bit_set(result, pin),
            PinSet::Low => bit_clear(result, pin),
        };

        self.write_config(Register::Gppu, port, result)
    }
}

impl<I2C, E> Interrupts for Mcp23017<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    fn find_interrupted_pin(&mut self, port: MyPort) -> PinNumber {
        let pin_msk = self.read_config(Register::Intf, port).unwrap();

        pin_mask_to_number(PinMask::from(pin_msk))
    }

    fn set_interrupt_on(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        interrupt_on: InterruptOn,
    ) -> Result<(), Error> {
        let mut reg = self.read_config(Register::Intcon, port)?;

        reg = match interrupt_on {
            InterruptOn::PinChange => bit_clear(reg, pin),
            InterruptOn::ChangeFromRegister => bit_set(reg, pin),
        };

        self.write_config(Register::Intcon, port, reg)
    }

    fn set_interrupt_compare(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        value: PinSet,
    ) -> Result<(), Error> {
        let intcon = self.read_config(Register::Intcon, port)?;

        if bit_read(intcon, pin) != 1 {
            return Err(Error::InvalidInterruptSetting);
        }

        let mut reg = self.read_config(Register::Defval, port)?; //change only valid if intcon is set to 1

        reg = match value {
            PinSet::High => bit_set(reg, pin),
            PinSet::Low => bit_clear(reg, pin),
        };

        self.write_config(Register::Defval, port, reg)
    }

    fn disable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
        let mut reg = self.read_config(Register::Gpinten, port)?;

        reg = bit_clear(reg, pin);

        self.write_config(Register::Gpinten, port, reg)
    }

    fn enable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
        //todo Set GPINTEN, only valid if DEFVAL and INTCON already configured
        let mut reg = self.read_config(Register::Gpinten, port)?;

        reg = bit_set(reg, pin);
        self.write_config(Register::Gpinten, port, reg)
    }

    fn set_mirror(&mut self, mirror: InterruptMirror) -> Result<(), Error> {
        let mut rega = self.read_config(Register::Iocon, MyPort::Porta)?;
        let mut regb = self.read_config(Register::Iocon, MyPort::Portb)?;

        match mirror {
            InterruptMirror::MirrorOn => {
                rega |= InterruptMirror::MirrorOn as u8;
                regb |= InterruptMirror::MirrorOn as u8;
            }
            InterruptMirror::MirrorOff => {
                rega &= !(InterruptMirror::MirrorOn as u8);
                regb &= !(InterruptMirror::MirrorOn as u8);
            }
        }

        self.write_config(Register::Iocon, MyPort::Porta, rega)?;

        self.write_config(Register::Iocon, MyPort::Portb, regb)
    }
}

#[allow(dead_code)]
impl<I2C, E> Mcp23017<I2C, Configuring>
where
    I2C: i2c::I2c<Error = E>,
{
    fn port_mode(self) -> Mcp23017<I2C, PortMode> {
        Mcp23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<PortMode>,
        }
    }

    fn pin_mode(self) -> Mcp23017<I2C, PinMode> {
        Mcp23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<PinMode>,
        }
    }

    fn chip_mode(self) -> Mcp23017<I2C, ChipMode> {
        Mcp23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<ChipMode>,
        }
    }
}

impl<I2C, E> MyOutput for Mcp23017<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    fn write(&mut self, value: u16) -> Result<(), Error> {
        let register_address = Register::Gpio as u8;
        let bytes = value.to_be_bytes();
        self.i2c
            .write(self.address, &[register_address, bytes[0], bytes[1]])
            .map_err(i2c_comm_error)?;
        Ok(())
    }

    fn write_port(&mut self, port: MyPort, value: u8) -> Result<(), Error> {
        let register_address = Register::Gpio as u8 | port as u8;
        self.i2c
            .write(self.address, &[register_address, value])
            .map_err(i2c_comm_error)?;
        Ok(())
    }

    fn write_pin(&mut self, port: MyPort, pin: PinNumber, value: PinSet) -> Result<(), Error> {
        let mut final_value = self.read_port(port).unwrap();

        final_value = match value {
            PinSet::High => bit_set(final_value, pin),
            PinSet::Low => bit_clear(final_value, pin),
        };

        self.write_port(port, final_value)
    }
}

impl<I2C, E> MyInput for Mcp23017<I2C>
where
    I2C: i2c::I2c<Error = E>,
{
    fn read(&mut self) -> Result<u16, Error> {
        let register_address = Register::Gpio as u8;
        let mut rx_buffer: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .map_err(i2c_comm_error)?;
        Ok(LittleEndian::read_u16(&rx_buffer))
    }

    fn read_port(&mut self, port: MyPort) -> Result<u8, Error> {
        let register_address = Register::Gpio as u8 | port as u8;
        let mut rx_buffer: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .map_err(i2c_comm_error)?;
        Ok(rx_buffer[0])
    }

    fn read_pin(&mut self, port: MyPort, pin: PinNumber) -> Result<u8, Error> {
        let mut result = self.read_port(port).unwrap();

        result = bit_read(result, pin);
        Ok(result)
    }
}

/////// Tests

#[cfg(test)]
mod tests {
    use std::println;

    use super::*;
    use crate::bit_read;
    extern crate std;

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
