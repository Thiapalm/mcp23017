#![allow(unused)]

use crate::prelude::*;
use crate::registers::*;
use MyPort::Porta as porta;
use MyPort::Portb as portb;

use byteorder::{ByteOrder, LittleEndian};
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), keep_self,),
    async(feature = "async", keep_self)
)]
trait Regread {
    async fn read_config(&mut self, register: Register) -> Result<u8, Error>;
    async fn write_config(&mut self, register: Register, value: u8) -> Result<(), Error>;
}

macro_rules! define_port {
    ($port_name: ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $port_name<I2C, State = Configuring> {
            i2c: I2C,
            address: u8,
            port: MyPort,
            state: core::marker::PhantomData<State>,
        }
    };
}

macro_rules! create_port {
    ($port_name: ident, $my_port: ident) => {
        impl<I2C, E, State> $port_name<I2C, State>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to create a new handler for chip/port/pin
             */
            #[inline]
            pub fn new(i2c: I2C, address: u8) -> Self {
                $port_name {
                    i2c,
                    address,
                    port: $my_port,
                    state: Default::default(),
                }
            }
        }
    };
}

macro_rules! read_write {
    ($port_name: ident, $port_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                    sync(cfg(not(feature = "async")), self = $port_literal,),
                                    async(feature = "async", keep_self)
                                )]
        impl<I2C, E, State> Regread for $port_name<I2C, State>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Private function used to read the chip registers using i2c
             */
            #[inline]
            async fn read_config(&mut self, register: Register) -> Result<u8, Error> {
                let register_address = register as u8 | self.port as u8;

                let mut rx_buffer: [u8; 1] = [0; 1];
                self.i2c
                    .write_read(self.address, &[register_address], &mut rx_buffer)
                    .await
                    .map_err(i2c_comm_error)?;
                Ok(rx_buffer[0])
            }

            /**
             * Private function used to write the chip registers using i2c
             */
            #[inline]
            async fn write_config(&mut self, register: Register, value: u8) -> Result<(), Error> {
                let register_address = register as u8 | self.port as u8;

                self.i2c
                    .write(self.address, &[register_address, value])
                    .await
                    .map_err(i2c_comm_error)?;
                Ok(())
            }
        }
    };
}

macro_rules! set_as {
    ($port_name: ident, $port_literal: literal) => {
        #[allow(dead_code)]
        #[maybe_async_cfg::maybe(
                                    sync(cfg(not(feature = "async")), self = $port_literal,),
                                    async(feature = "async", keep_self)
                                )]
        impl<I2C, E> $port_name<I2C, Configuring>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to set the chip/port/pin as input
             */
            #[inline]
            pub async fn set_as_input(
                mut self,
            ) -> Result<$port_name<I2C, InputConfiguring>, Error> {
                self.write_config(Register::Iodir, 0xFF)
                    .await?;

                Ok($port_name {
                    i2c: self.i2c,
                    address: self.address,
                    port: self.port,
                    state: core::marker::PhantomData::<InputConfiguring>,
                })
            }

            /**
             * Function used to set the chip/port/pin as output
             */
            #[inline]
            pub async fn set_as_output(mut self) -> Result<$port_name<I2C, OutputReady>, Error> {
                self.write_config(Register::Iodir, 0x00)
                    .await?;

                Ok($port_name {
                    i2c: self.i2c,
                    address: self.address,
                    port: self.port,
                    state: core::marker::PhantomData::<OutputReady>,
                })
            }
        }
    };
}

macro_rules! outputready {
    ($port_name: ident, $port_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                        sync(cfg(not(feature = "async")), self = $port_literal,),
                                        async(feature = "async", keep_self)
                                    )]
        impl<I2C, E> $port_name<I2C, OutputReady>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to write the output value to be set on chip/port/pin
             */
            #[inline]
            pub async fn write(&mut self, value: u8) -> Result<(), Error> {
                let register_address = Register::Gpio as u8 | self.port as u8;
                self.write_config(Register::Gpio, value).await?;

                Ok(())
            }

            /**
             * Function used to write the output value to be set on pin
             */
            #[inline]
            pub async fn write_pin(&mut self, pin: PinNumber, value: PinSet) -> Result<(), Error> {
                let mut result = self.read_config(Register::Gpio).await?;

                result = match value {
                    PinSet::High => bit_set(result, pin),
                    PinSet::Low => bit_clear(result, pin),
                };

                self.write_config(Register::Gpio, result).await.map_err(i2c_comm_error)?;

                Ok(())
            }
        }
    };
}

macro_rules! inputready {
    ($port_name: ident, $port_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                    sync(cfg(not(feature = "async")), self = $port_literal,),
                                    async(feature = "async", keep_self)
                                )]
        impl<I2C, E> $port_name<I2C, InputReady>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to read the input
             */
            #[inline]
            pub async fn read(&mut self) -> Result<u8, Error> {

                let mut result = self.read_config(Register::Gpio).await.map_err(i2c_comm_error)?;

                Ok(result)
            }

            /**
             * Function used to read the input pin
             */
            #[inline]
            pub async fn read_pin(&mut self, pin: PinNumber) -> Result<u8, Error> {
                let result = self.read().await?;
                Ok(bit_read(result, pin))
            }

            /**
             * Function used to disable the interrupt on the input
             */
            #[inline]
            pub async fn disable_interrupt(&mut self, pin: PinNumber) -> Result<(), Error> {
                let mut reg = self.read_config(Register::Gpinten).await?;

                reg = bit_clear(reg, pin);

                self.write_config(Register::Gpinten, reg).await
            }

            /**
             * Function used to enable the interrupt on the input
             */
            #[inline]
            pub async fn enable_interrupt(
                &mut self,
                pin: PinNumber,
            ) -> Result<(), Error> {
                let mut reg = self.read_config(Register::Gpinten).await?;

                reg = bit_set(reg, pin);
                self.write_config(Register::Gpinten, reg).await
            }

            /**
             * Function used to verify the interrupt on the input
             */
            #[inline]
            pub async fn get_interrupted_pin(&mut self) -> Option<PinNumber> {
                let pin_msk = self.read_config(Register::Intf).await.unwrap_or(0);

                pin_mask_to_number(PinMask::from(pin_msk))
            }
        }
    };
}

macro_rules! inputconfiguring {
    ($port_name: ident, $port_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                    sync(cfg(not(feature = "async")), self = $port_literal,),
                                    async(feature = "async", keep_self)
                                )]
        impl<I2C, E> $port_name<I2C, InputConfiguring>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to set the pull on the input
             */
            #[inline]
            pub async fn set_pull(mut self, pull: PinSet) -> Result<Self, Error> {
                let result = match pull {
                    PinSet::High => 0xFF,
                    PinSet::Low => 0x00,
                };

                self.write_config(Register::Gppu, result).await?;

                Ok(self)
            }

            /**
             * Function used to set the interrupt mirror function on the input
             */
            #[inline]
            pub async fn set_interrupt_mirror(
                mut self,
                mirror: InterruptMirror,
            ) -> Result<Self, Error> {
                let mut reg = self.read_config(Register::Iocon).await?;

                match mirror {
                    InterruptMirror::MirrorOn => {
                        reg |= InterruptMirror::MirrorOn as u8;
                    }
                    InterruptMirror::MirrorOff => {
                        reg &= !(InterruptMirror::MirrorOn as u8);
                    }
                }

                self.write_config(Register::Iocon, reg)
                    .await?;

                Ok(self)
            }

            /**
             * Function used to choose the pin as interrupt on the input
             */
            #[inline]
            pub async fn set_interrupt_on(
                mut self,
                pin: PinNumber,
                interrupt_on: InterruptOn,
            ) -> Result<Self, Error> {
                let mut reg = self.read_config(Register::Intcon).await?;

                reg = match interrupt_on {
                    InterruptOn::PinChange => bit_clear(reg, pin),
                    InterruptOn::ChangeFromRegister => bit_set(reg, pin),
                };

                self.write_config(Register::Intcon, reg).await?;
                Ok(self)
            }

            /**
             * Function used to set the interrupt compare function on the input
             */
            #[inline]
            pub async fn set_interrupt_compare(
                mut self,
                pin: PinNumber,
                value: PinSet,
            ) -> Result<Self, Error> {
                let intcon = self.read_config(Register::Intcon).await?;

                if bit_read(intcon, pin) != 1 {
                    return Err(Error::InvalidInterruptSetting);
                }

                let mut reg = self.read_config(Register::Defval).await?; //change only valid if intcon is set to 1

                reg = match value {
                    PinSet::High => bit_set(reg, pin),
                    PinSet::Low => bit_clear(reg, pin),
                };

                self.write_config(Register::Defval, reg).await?;
                Ok(self)
            }

            /**
             * Function used to set input to the ready state
             */
            #[inline]
            pub fn ready(mut self) -> $port_name<I2C, InputReady> {
                $port_name {
                    i2c: self.i2c,
                    address: self.address,
                    port: self.port,
                    state: core::marker::PhantomData::<InputReady>,
                }
            }
        }
    };
}

define_port!(PortA);
create_port!(PortA, porta);
read_write!(PortA, "PortA");
set_as!(PortA, "PortA");
outputready!(PortA, "PortA");
inputconfiguring!(PortA, "PortA");
inputready!(PortA, "PortA");

define_port!(PortB);
create_port!(PortB, portb);
read_write!(PortB, "PortB");
set_as!(PortB, "PortB");
outputready!(PortB, "PortB");
inputconfiguring!(PortB, "PortB");
inputready!(PortB, "PortB");

#[cfg(test)]
mod tests {
    extern crate std;
    use core::marker::PhantomData;

    use super::*;
    use embedded_hal::i2c::ErrorKind;
    use pretty_assertions::assert_eq;
    extern crate embedded_hal_mock;
    use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use tests::std::vec::Vec;

    fn vector1(a: u8) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(a);
        v
    }
    fn vector2(a: u8, b: u8) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(a);
        v.push(b);
        v
    }
    fn vector3(a: u8, b: u8, c: u8) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(a);
        v.push(b);
        v.push(c);
        v
    }

    #[test]
    fn test_read_config_porta() {
        let expectations = [I2cTransaction::write_read(
            0x40,
            vector1(Register::Gpio as u8 | MyPort::Porta as u8),
            vector1(0xff),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut myporta: PortA<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            PortA::new(i2c.clone(), 0x40);
        let result = myporta.read_config(Register::Gpio);
        assert_eq!(0xff, result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_config_portb() {
        let expectations = [I2cTransaction::write_read(
            0x40,
            vector1(Register::Gpio as u8 | MyPort::Portb as u8),
            vector1(0xff),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut myportb: PortB<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            PortB::new(i2c.clone(), 0x40);
        let result = myportb.read_config(Register::Gpio);
        assert_eq!(0xff, result.unwrap());

        //finalize execution
        i2c.done();
    }
}
