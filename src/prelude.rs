#![allow(dead_code, unused)]

use crate::registers::*;
use core::fmt::Display;

const DEFAULT_ADDRESS: u8 = 0x20;

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

///Valid error codes
#[derive(Clone, Copy, Debug, PartialEq)]
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
