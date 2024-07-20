# `mcp23017`

> 16-Bit I/O Expander with Serial Interface

<p align=center>
  <a href="https://crates.io/crates/mcp23017-tp"><img src="https://img.shields.io/badge/crates.io-v0.1.1-red"></a>
 <a href="https://docs.rs/mcp23017-tp/0.1.1/mcp23017_tp/"><img src="https://img.shields.io/badge/docs.rs-v0.1.1-orange"></a>
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

NOTE: When operating in 16bit mode, use LittleEndian formatting (0xbbaa).

# Version Revision

0.1.0 - First Version

0.1.1 - Fixed doc generation, fixed async support in traits, improved prelude for feature usage, fixed endianess for interrupt functions

# Features 

features = ["async"] - enables support for async Rust (Currently embedded_hal_bus does not implement async for I2C, therefore if using more than one pin/port, disable the async feature)

features = ["chipmode"] - The driver operates as a 1x 16bit device set entirely as output or input

features = ["portmode"] - The driver operates as a 2x 8bit port device, each port is configured individually

features = ["pinmode"] - The driver operates as a 16x 1bit pins device, each pin is configured individually

ATTENTION: ENABLE ONLY ONE OF THE MODES OR FACE THE CONSEQUENCES.... ASYNC CAN BE USED ON ANY MODE

# Example

To use the driver, you must have a concrete implementation of the
[embedded-hal](https://crates.io/crates/embedded-hal) traits.  This example uses
[stm32f4xx-hal](https://crates.io/crates/stm32f4xx-hal):


When using chipmode, the driver will operate in 16bit, the code below will set all pins to output:

``` rust
use core::cell::RefCell;
use embedded_hal_bus::i2c;
use mcp23017_tp::prelude::*;

    let mut i2c = dp.I2C1.i2c(
        (scl, sda),
        Mode::Standard {
            frequency: 100.kHz(),
        },
        &clocks,
    );

    let i2c_ref_cell = RefCell::new(i2c);

    let mut mcp = mcp23017_tp::MCP23017::new(i2c::RefCellDevice::new(&i2c_ref_cell), address)
        .set_as_output()
        .unwrap();

    loop {
          mcp.write(0xbbaa).unwrap();
          delay.delay_ms(2000);

          // u16: 0xbbaa - u8[]: [0]aa [1]bb (LittleEndian)
          mcp.write(0x0000).unwrap();
          delay.delay_ms(2000);
        }
```

When using portmode, the driver will operate in 2x8bit, the code below will set port A as output and port B as input:

``` rust
use core::cell::RefCell;
use embedded_hal_bus::i2c;
use mcp23017_tp::prelude::*;

    let mut i2c = dp.I2C1.i2c(
        (scl, sda),
        Mode::Standard {
            frequency: 100.kHz(),
        },
        &clocks,
    );

    let i2c_ref_cell = RefCell::new(i2c);

    let mut porta = mcp23017_tp::PortA::new(i2c::RefCellDevice::new(&i2c_ref_cell), address)
         .set_as_output()
         .unwrap();
    
    let mut portb = mcp23017_tp::PortB::new(i2c::RefCellDevice::new(&i2c_ref_cell), address)
        .set_as_input()
        .unwrap()
        .set_pull(PinSet::High)
        .unwrap()
        .ready();

    loop {
          porta.write(0xff).unwrap();
          delay.delay_ms(2000);
          porta.write(0x00).unwrap();
          delay.delay_ms(2000);

          rprintln!("{:#02x}", portb.read().unwrap());
        }
```

When using pinmode, the driver will operate in 16x1bit, the code below will set pin A1 and pin B3 as input:

``` rust
use core::cell::RefCell;
use embedded_hal_bus::i2c;
use mcp23017_tp::prelude::*;

    let mut i2c = dp.I2C1.i2c(
        (scl, sda),
        Mode::Standard {
            frequency: 100.kHz(),
        },
        &clocks,
    );

    let i2c_ref_cell = RefCell::new(i2c);

    let mut pina1 = mcp23017_tp::Pina1::new(i2c::RefCellDevice::new(&i2c_ref_cell), address)
        .set_as_input()
        .unwrap()
        .set_pull(PinSet::High)
        .unwrap()
        .ready();

    let mut pinb3 = mcp23017_tp::Pinb3::new(i2c::RefCellDevice::new(&i2c_ref_cell), address)
        .set_as_input()
        .unwrap()
        .set_pull(PinSet::High)
        .unwrap()
        .ready();

    loop {
          delay.delay_ms(2000);
          rprintln!(
              "{:#02x} {:#02x}",
              pina1.read().unwrap(),
              pinb3.read().unwrap()
          );
        }
```

# License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.