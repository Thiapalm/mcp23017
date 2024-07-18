# `mcp23017`

> 16-Bit I/O Expander with Serial Interface

<p align=center>
  <a href="https://crates.io/crates/mcp23017-tp"><img src="https://img.shields.io/badge/crates.io-v0.1.0-red"></a>
 <a href="https://docs.rs/mcp23017-tp/0.3.0/mcp23017-tp/"><img src="https://img.shields.io/badge/docs.rs-v0.1.0-orange"></a>
 <a href="http://www.apache.org/licenses/LICENSE-2.0"><img src="https://img.shields.io/badge/License-ApacheV2-green"></a>
 <a href="http://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-green"></a>
</p>

# [mcp23017 Datasheet](https://ww1.microchip.com/downloads/aemDocuments/documents/APID/ProductDocuments/DataSheets/MCP23017-Data-Sheet-DS20001952.pdf)

# Description

This crate was made for and tested on MCP23017 from Microchip, it is based on I2C from embedded-hal crate.
The implementation of this crate is based on #![no_std] but with some minor adjustments it can be used on std environments.

This driver allows you to:
- choose operating mode: (1x16bit), (2x8bit) or (16x1bit)
- configure interrupts
- enable or disable interrupts
- set internall pull resistor
- read or write to pin/port/chip dependiong on the mode choosen


# Version Revision

0.1.0 - First Version

# Features 

features = ["async"] - enables support for async Rust

features = ["chipmode"] - The driver operates as a 1x 16bit device set entirely as output or input

features = ["portmode"] - The driver operates as a 2x 8bit port device

features = ["pinmode"] - The driver operates as a 16x 1bit pins device configured individually

ATTENTION: ENABLE ONLY ONE OF THE MODES OR FACE THE CONSEQUENCES.... ASYNC CAN BE USED ON ANY MODE

# Example

To use the driver, you must have a concrete implementation of the
[embedded-hal](https://crates.io/crates/embedded-hal) traits.  This example uses
[stm32f4xx-hal](https://crates.io/crates/stm32f4xx-hal):

