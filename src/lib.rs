#![no_std]

// Imports
use byteorder::{BigEndian, ByteOrder};
use core::fmt::Display;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use rtt_target::rprintln;

const DEFAULT_ADDRESS: u8 = 0x20; // Default address

#[allow(dead_code)]
enum Register {
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
pub enum PinNumber {
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
pub enum MyPort {
    Porta = 0x00,
    Portb = 0x01,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Output,
    Input,
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NonConfigured;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Output;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct InputPullUp;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Input;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Pin<State = NonConfigured> {
    port: MyPort,
    address: Option<u8>,
    mask: u8,
    state: core::marker::PhantomData<State>,
}

#[cfg(feature = "bit")]
impl<State> Pin<State> {
    pub fn clip(&mut self) -> &mut Self {
        self
    }
}

#[cfg(feature = "bit")]
impl Pin<Output> {
    pub fn high<I2C, E>(&mut self, i2c: &mut I2C) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let mut buffer = self
            .read(i2c, (Register::Olat as u8) | (self.port as u8))
            .unwrap();
        rprintln!(
            "Read from Olat: {:#04x} value {:#04x}",
            (Register::Olat as u8) | (self.port as u8),
            buffer[0]
        );

        buffer[0] = bit_set(buffer[0], self.mask);

        rprintln!(
            "Writing to: {:#04x} value {:#04x}",
            (Register::Gpio as u8 | self.port as u8),
            buffer[0]
        );

        let result = i2c.write(
            self.address.unwrap(),
            &[(Register::Gpio as u8 | self.port as u8), buffer[0]],
        );

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::CommunicationErr),
        }
    }
    pub fn low<I2C, E>(&mut self, i2c: &mut I2C) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let buffer = self.read(i2c, (Register::Olat as u8) | (self.port as u8));

        let mut buffer = match buffer {
            Ok(x) => x,
            Err(x) => {
                rprintln!("Error: {:?}", x);
                return Err(x);
            }
        };
        rprintln!(
            "Read from Olat: {:#04x} value {:#04x}",
            (Register::Olat as u8) | (self.port as u8),
            buffer[0]
        );

        buffer[0] = bit_clear(buffer[0], self.mask);

        let result = i2c.write(
            self.address.unwrap(),
            &[(Register::Gpio as u8 | self.port as u8), buffer[0]],
        );

        rprintln!(
            "Writing to: {:#04x} value {:#04x}",
            (Register::Gpio as u8 | self.port as u8),
            buffer[0]
        );

        match result {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::CommunicationErr),
        }
    }
}

#[cfg(feature = "bit")]
impl Pin<NonConfigured> {
    fn new(pin_number: PinNumber, port: MyPort) -> Self {
        Pin {
            address: None,
            port,
            mask: pin_number as u8,
            state: core::marker::PhantomData::<NonConfigured>,
        }
    }

    fn set_address(&mut self, address: u8) -> &mut Self {
        self.address = Some(address);
        self
    }

    pub fn set_pin_output<I2C, E>(&mut self, i2c: &mut I2C) -> Pin<Output>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        rprintln!(
            "Set pin output => address: {:#04x} port: {:#04x} mask: {:#04x}",
            self.address.unwrap(),
            self.port as u8,
            self.mask
        );

        let mut mypin = Pin {
            address: self.address,
            port: self.port,
            mask: self.mask,
            state: core::marker::PhantomData::<Output>,
        };

        let result = mypin.read(i2c, (Register::Iodir as u8) | (self.port as u8));

        if let Ok(mut result) = result {
            rprintln!(
                "Reading from Iodir {:#04x}) value {:#04x}",
                (Register::Iodir as u8) | (self.port as u8),
                result[0]
            );
            result[0] = bit_clear(result[0], self.mask);

            let _ = mypin.write(i2c, (Register::Iodir as u8) | (self.port as u8), result[0]);
            rprintln!(
                "Writing to Iodir {:#04x}) value {:#04x}",
                (Register::Iodir as u8) | (self.port as u8),
                result[0]
            );
            result[0] = bit_clear(result[0], self.mask);
        }
        mypin
    }

    pub fn set_pin_input_pullup<I2C, E>(&mut self, i2c: &mut I2C) -> Pin<InputPullUp>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let mut mypin = Pin {
            address: self.address,
            port: self.port,
            mask: self.mask,
            state: core::marker::PhantomData::<InputPullUp>,
        };

        //set to Input
        let mut result = mypin.read(i2c, (Register::Iodir as u8) | (self.port as u8));

        if let Ok(mut result) = result {
            result[0] = bit_clear(result[0], self.mask);

            mypin.write(i2c, (Register::Iodir as u8) | (self.port as u8), result[0]);
        }

        //Set pullup
        let mut result = mypin.read(i2c, (Register::Gppu as u8) | (self.port as u8));

        if let Ok(mut result) = result {
            result[0] = bit_set(result[0], self.mask);

            mypin.write(i2c, (Register::Gppu as u8) | (self.port as u8), result[0]);
        }

        mypin
    }

    pub fn set_pin_input<I2C, E>(&mut self, i2c: &mut I2C) -> Pin<Input>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let mut mypin = Pin {
            address: self.address,
            port: self.port,
            mask: self.mask,
            state: core::marker::PhantomData::<Input>,
        };

        //set to Input
        let mut result = mypin.read(i2c, (Register::Iodir as u8) | (self.port as u8));

        if let Ok(mut result) = result {
            result[0] = bit_clear(result[0], self.mask);

            mypin.write(i2c, (Register::Iodir as u8) | (self.port as u8), result[0]);
        }
        mypin
    }
}

fn bit_set(byte: u8, mask: u8) -> u8 {
    byte | mask
}

fn bit_clear(byte: u8, mask: u8) -> u8 {
    byte & !mask
}

#[cfg(feature = "port")]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Port<State = NonConfigured> {
    pin0: Pin<State>,
    pin1: Pin<State>,
    pin2: Pin<State>,
    pin3: Pin<State>,
    pin4: Pin<State>,
    pin5: Pin<State>,
    pin6: Pin<State>,
    pin7: Pin<State>,
    state: core::marker::PhantomData<State>,
}

#[cfg(feature = "port")]
impl Port<NonConfigured> {
    pub fn new() -> Self {
        Port {
            pin0: Pin::new(),
            pin1: Pin::new(),
            pin2: Pin::new(),
            pin3: Pin::new(),
            pin4: Pin::new(),
            pin5: Pin::new(),
            pin6: Pin::new(),
            pin7: Pin::new(),
            state: core::marker::PhantomData::<NonConfigured>,
        }
    }

    pub fn port_as_output(&mut self) -> Result<Port<Output>, Error> {
        let port = Port {
            state: core::marker::PhantomData::<Output>,
            pin0: Pin::new().set_pin_output(),
            pin1: Pin::new().set_pin_output(),
            pin2: Pin::new().set_pin_output(),
            pin3: Pin::new().set_pin_output(),
            pin4: Pin::new().set_pin_output(),
            pin5: Pin::new().set_pin_output(),
            pin6: Pin::new().set_pin_output(),
            pin7: Pin::new().set_pin_output(),
        };
        Ok(port)
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
    fn write(&mut self, i2c: &mut I2C, register: u8, buf: u8) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>;
    fn read(&mut self, i2c: &mut I2C, register: u8) -> Result<[u8; 1], Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>;
}

//todo 1. Faça funcionar
//todo 2. generalize
//todo 3. imponha restrições

#[cfg(feature = "bit")]
pub struct Mcp23017 {
    pina0: Pin,
    pina1: Pin,
    pina2: Pin,
    pina3: Pin,
    pina4: Pin,
    pina5: Pin,
    pina6: Pin,
    pina7: Pin,
    pinb0: Pin,
    pinb1: Pin,
    pinb2: Pin,
    pinb3: Pin,
    pinb4: Pin,
    pinb5: Pin,
    pinb6: Pin,
    pinb7: Pin,
}

#[cfg(feature = "port")]
pub struct Mcp23017 {
    address: Option<u8>,
    porta: Port,
    portb: Port,
}

impl Default for Mcp23017 {
    fn default() -> Self {
        Self::new(Some(DEFAULT_ADDRESS))
    }
}

impl Mcp23017 {
    #[cfg(feature = "port")]
    pub fn new(address: Option<u8>) -> Self {
        Mcp23017 {
            address,
            porta: Port::new(),
            portb: Port::new(),
        }
    }
    #[cfg(feature = "bit")]
    pub fn new(address: Option<u8>) -> Self {
        Mcp23017 {
            pina0: Pin::new(PinNumber::Pin0, MyPort::Porta),
            pina1: Pin::new(PinNumber::Pin1, MyPort::Porta),
            pina2: Pin::new(PinNumber::Pin2, MyPort::Porta),
            pina3: Pin::new(PinNumber::Pin3, MyPort::Porta),
            pina4: Pin::new(PinNumber::Pin4, MyPort::Porta),
            pina5: Pin::new(PinNumber::Pin5, MyPort::Porta),
            pina6: Pin::new(PinNumber::Pin6, MyPort::Porta),
            pina7: Pin::new(PinNumber::Pin7, MyPort::Porta),
            pinb0: Pin::new(PinNumber::Pin0, MyPort::Portb),
            pinb1: Pin::new(PinNumber::Pin1, MyPort::Portb),
            pinb2: Pin::new(PinNumber::Pin2, MyPort::Portb),
            pinb3: Pin::new(PinNumber::Pin3, MyPort::Portb),
            pinb4: Pin::new(PinNumber::Pin4, MyPort::Portb),
            pinb5: Pin::new(PinNumber::Pin5, MyPort::Portb),
            pinb6: Pin::new(PinNumber::Pin6, MyPort::Portb),
            pinb7: Pin::new(PinNumber::Pin7, MyPort::Portb),
        }
    }

    pub fn set_address(&mut self, address: u8) -> &mut Self {
        self.pina0.set_address(address);
        self.pina1.set_address(address);
        self.pina2.set_address(address);
        self.pina3.set_address(address);
        self.pina4.set_address(address);
        self.pina5.set_address(address);
        self.pina6.set_address(address);
        self.pina7.set_address(address);

        self.pinb0.set_address(address);
        self.pinb1.set_address(address);
        self.pinb2.set_address(address);
        self.pinb3.set_address(address);
        self.pinb4.set_address(address);
        self.pinb5.set_address(address);
        self.pinb6.set_address(address);
        self.pinb7.set_address(address);

        self
    }

    pub fn find_me<I2C, E>(&mut self, address: u8, i2c: &mut I2C) -> Result<(), Error>
    where
        I2C: WriteRead<Error = E> + Write<Error = E>,
    {
        let mut rx_buffer: [u8; 1] = [0; 1];
        i2c.write_read(
            address,
            &[Register::Iodir as u8 | MyPort::Porta as u8],
            &mut rx_buffer,
        )
        .map_err(i2c_comm_error)?;

        // rprintln!("defval: {:?}", rx_buffer);
        rprintln!(
            "Iodira {:#04x} has value {:#02x}",
            Register::Iodir as u8 | MyPort::Porta as u8,
            rx_buffer[0]
        );
        i2c.write_read(
            address,
            &[Register::Iodir as u8 | MyPort::Portb as u8],
            &mut rx_buffer,
        )
        .map_err(i2c_comm_error)?;

        rprintln!(
            "Iodirb {:#04x} has value {:#02x}",
            Register::Iodir as u8 | MyPort::Portb as u8,
            rx_buffer[0]
        );

        rx_buffer[0] = 128;

        let _ = i2c.write(address, &[Register::Iocon as u8, rx_buffer[0]]);

        Ok(())
    }

    pub fn split(&mut self, pin: PinNumber, port: MyPort) -> &mut Pin<NonConfigured> {
        match (port, pin) {
            (MyPort::Porta, PinNumber::Pin0) => &mut self.pina0,
            (MyPort::Porta, PinNumber::Pin1) => &mut self.pina1,
            (MyPort::Porta, PinNumber::Pin2) => &mut self.pina2,
            (MyPort::Porta, PinNumber::Pin3) => &mut self.pina3,
            (MyPort::Porta, PinNumber::Pin4) => &mut self.pina4,
            (MyPort::Porta, PinNumber::Pin5) => &mut self.pina5,
            (MyPort::Porta, PinNumber::Pin6) => &mut self.pina6,
            (MyPort::Porta, PinNumber::Pin7) => &mut self.pina7,
            (MyPort::Portb, PinNumber::Pin0) => &mut self.pinb0,
            (MyPort::Portb, PinNumber::Pin1) => &mut self.pinb1,
            (MyPort::Portb, PinNumber::Pin2) => &mut self.pinb2,
            (MyPort::Portb, PinNumber::Pin3) => &mut self.pinb3,
            (MyPort::Portb, PinNumber::Pin4) => &mut self.pinb4,
            (MyPort::Portb, PinNumber::Pin5) => &mut self.pinb5,
            (MyPort::Portb, PinNumber::Pin6) => &mut self.pinb6,
            (MyPort::Portb, PinNumber::Pin7) => &mut self.pinb7,
        }
    }
}

impl<I2C, E, State> Operation<I2C, E> for Pin<State>
where
    I2C: WriteRead<Error = E> + Write<Error = E>,
{
    fn write(&mut self, i2c: &mut I2C, register: u8, buf: u8) -> Result<(), Error> {
        match self.address {
            Some(address) => {
                let _ = i2c.write(address, &[register, buf]);
                Ok(())
            }
            None => Err(Error::MissingAddress),
        }
    }

    fn read(&mut self, i2c: &mut I2C, register: u8) -> Result<[u8; 1], Error> {
        let mut rx_buffer: [u8; 1] = [0; 1];

        match self.address {
            Some(address) => {
                i2c.write_read(address, &[register], &mut rx_buffer)
                    .map_err(i2c_comm_error)?;
                Ok(rx_buffer)
            }
            None => Err(Error::MissingAddress),
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pin_test() {
        let mut pin0 = Pin::new(PinNumber::Pin0, MyPort::Porta);
        assert_eq!(pin0.state, core::marker::PhantomData::<NonConfigured>);
        let pin0 = pin0.set_pin_output();
        assert_eq!(pin0.state, core::marker::PhantomData::<Output>);
        let pin1 = Pin::new(PinNumber::Pin1, MyPort::Porta).set_pin_input_pullup();
        assert_eq!(pin1.state, core::marker::PhantomData::<InputPullUp>);
        let pin2 = Pin::new(PinNumber::Pin2, MyPort::Porta).set_pin_input();
        assert_eq!(pin2.state, core::marker::PhantomData::<Input>);
    }

    #[test]
    fn new_mcp23017_test() {
        let mut mcp = Mcp23017::new(None);
        let mut pina = mcp.pina1.set_pin_output().clip();
    }
}
