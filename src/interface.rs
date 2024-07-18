#![allow(unused)]
use crate::registers::*;

pub trait RegReadWrite {
    fn write_config(&mut self, register: Register, value: u16) -> Result<(), Error>;
    fn read_config(&mut self, register: Register) -> Result<u16, Error>;
}
/////// Traits
pub trait Configuration {
    fn set_pin_dir(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        direction: Direction,
    ) -> Result<(), Error>;
    fn set_pull(&mut self, port: MyPort, pin: PinNumber, pull: PinSet) -> Result<(), Error>; //set GPPU (1 up, 0 disabled) return error if pin is output
}

pub trait Interrupts {
    fn find_interrupted_pin(&mut self, port: MyPort) -> Option<PinNumber>; //Read INTF register
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
