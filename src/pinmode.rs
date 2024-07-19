#![allow(unused)]

use crate::prelude::*;
use crate::registers::*;
use MyPort::Porta as porta;
use MyPort::Portb as portb;

use PinNumber::Pin0 as pin0;
use PinNumber::Pin1 as pin1;
use PinNumber::Pin2 as pin2;
use PinNumber::Pin3 as pin3;
use PinNumber::Pin4 as pin4;
use PinNumber::Pin5 as pin5;
use PinNumber::Pin6 as pin6;
use PinNumber::Pin7 as pin7;

use byteorder::{ByteOrder, LittleEndian};
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

trait Regread {
    fn read_config(&mut self, register: Register) -> Result<u8, Error>;
    fn write_config(&mut self, register: Register, value: u8) -> Result<(), Error>;
}

macro_rules! define_pin {
    ($pin_name: ident) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $pin_name<I2C, State = Configuring> {
            i2c: I2C,
            address: u8,
            port: MyPort,
            pin: PinNumber,
            state: core::marker::PhantomData<State>,
        }
    };
}

macro_rules! create_pin {
    ($pin_name: ident, $my_port: ident, $my_pinnumber: ident) => {
        impl<I2C, E, State> $pin_name<I2C, State>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to create a new handler for chip/port/pin
             */
            #[inline]
            pub fn new(i2c: I2C, address: u8) -> Self {
                $pin_name {
                    i2c,
                    address,
                    port: $my_port,
                    pin: $my_pinnumber,
                    state: Default::default(),
                }
            }
        }
    };
}

macro_rules! read_write {
    ($pin_name: ident, $port_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                    sync(cfg(not(feature = "async")), self = $port_literal,),
                                    async(feature = "async", keep_self)
                                )]
        impl<I2C, E, State> Regread for $pin_name<I2C, State>
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
    ($pin_name: ident, $pin_literal: literal) => {
        #[allow(dead_code)]
        #[maybe_async_cfg::maybe(
                                            sync(cfg(not(feature = "async")), self = $pin_literal,),
                                            async(feature = "async", keep_self)
                                        )]
        impl<I2C, E> $pin_name<I2C, Configuring>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to set the chip/port/pin as input
             */
            #[inline]
            pub async fn set_as_input(mut self) -> Result<$pin_name<I2C, InputConfiguring>, Error> {
                let result = self.read_config(Register::Iodir).await?;
                self.write_config(Register::Iodir, bit_set(result, self.pin))
                    .await?;

                Ok($pin_name {
                    i2c: self.i2c,
                    address: self.address,
                    port: self.port,
                    pin: self.pin,
                    state: core::marker::PhantomData::<InputConfiguring>,
                })
            }

            /**
             * Function used to set the chip/port/pin as output
             */
            #[inline]
            pub async fn set_as_output(mut self) -> Result<$pin_name<I2C, OutputReady>, Error> {
                let result = self.read_config(Register::Iodir).await?;
                self.write_config(Register::Iodir, bit_clear(result, self.pin))
                    .await?;

                Ok($pin_name {
                    i2c: self.i2c,
                    address: self.address,
                    port: self.port,
                    pin: self.pin,
                    state: core::marker::PhantomData::<OutputReady>,
                })
            }
        }
    };
}

macro_rules! outputready {
    ($pin_name: ident, $pin_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                        sync(cfg(not(feature = "async")), self = $pin_literal,),
                                        async(feature = "async", keep_self)
                                    )]
        impl<I2C, E> $pin_name<I2C, OutputReady>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to write the output value to be set on chip/port/pin
             */
            #[inline]
            pub async fn write(&mut self, value: PinSet) -> Result<(), Error> {
                let mut result = self.read_config(Register::Gpio).await?;

                result = match value {
                    PinSet::High => bit_set(result, self.pin),
                    PinSet::Low => bit_clear(result, self.pin),
                };

                self.write_config(Register::Gpio, result).await.map_err(i2c_comm_error)?;

                Ok(())
            }
        }
    };
}

macro_rules! inputready {
    ($pin_name: ident, $pin_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                            sync(cfg(not(feature = "async")), self = $pin_literal,),
                                            async(feature = "async", keep_self)
                                        )]
        impl<I2C, E> $pin_name<I2C, InputReady>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to read the input
             */
            #[inline]
            pub async fn read(&mut self) -> Result<u8, Error> {
                let mut result = self.read_config(Register::Gpio).await?;

                Ok(bit_read(result, self.pin))
            }

            /**
             * Function used to disable the interrupt on the input
             */
            #[inline]
            pub async fn disable_interrupt(&mut self) -> Result<(), Error> {
                let mut reg = self.read_config(Register::Gpinten).await?;

                reg = bit_clear(reg, self.pin);

                self.write_config(Register::Gpinten, reg).await
            }

            /**
             * Function used to enable the interrupt on the input
             */
            #[inline]
            pub async fn enable_interrupt(&mut self) -> Result<(), Error> {
                let mut reg = self.read_config(Register::Gpinten).await?;

                reg = bit_set(reg, self.pin);
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

//TODO How to configure interrupt for each pin?
macro_rules! inputconfiguring {
    ($pin_name: ident, $pin_literal: literal) => {
        #[maybe_async_cfg::maybe(
                                            sync(cfg(not(feature = "async")), self = $pin_literal,),
                                            async(feature = "async", keep_self)
                                        )]
        impl<I2C, E> $pin_name<I2C, InputConfiguring>
        where
            I2C: I2c<Error = E>,
        {
            /**
             * Function used to set the pull on the input
             */
            #[inline]
            pub async fn set_pull(mut self, pull: PinSet) -> Result<Self, Error> {
                let mut reg = self.read_config(Register::Gppu).await?;

                reg = match pull {
                    PinSet::High => {bit_set(reg, self.pin)},
                    PinSet::Low => {bit_clear(reg, self.pin)}
                };

                self.write_config(Register::Gppu, reg).await?;

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

                self.write_config(Register::Iocon, reg).await?;

                Ok(self)
            }

            /**
             * Function used to choose the pin as interrupt on the input
             */
            #[inline]
            pub async fn set_interrupt_on(
                mut self,
                interrupt_on: InterruptOn,
            ) -> Result<Self, Error> {
                let mut reg = self.read_config(Register::Intcon).await?;

                reg = match interrupt_on {
                    InterruptOn::PinChange => bit_clear(reg, self.pin),
                    InterruptOn::ChangeFromRegister => bit_set(reg, self.pin),
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
                value: PinSet,
            ) -> Result<Self, Error> {
                let intcon = self.read_config(Register::Intcon).await?;

                if bit_read(intcon, self.pin) != 1 {
                    return Err(Error::InvalidInterruptSetting);
                }

                let mut reg = self.read_config(Register::Defval).await?; //change only valid if intcon is set to 1

                reg = match value {
                    PinSet::High => bit_set(reg, self.pin),
                    PinSet::Low => bit_clear(reg, self.pin),
                };

                self.write_config(Register::Defval, reg).await?;
                Ok(self)
            }

            /**
             * Function used to set input to the ready state
             */
            #[inline]
            pub fn ready(mut self) -> $pin_name<I2C, InputReady> {
                $pin_name {
                    i2c: self.i2c,
                    address: self.address,
                    port: self.port,
                    pin: self.pin,
                    state: core::marker::PhantomData::<InputReady>,
                }
            }
        }
    };
}

define_pin!(Pina0);
create_pin!(Pina0, porta, pin0);
read_write!(Pina0, "Pina0");
set_as!(Pina0, "Pina0");
outputready!(Pina0, "Pina0");
inputconfiguring!(Pina0, "Pina0");
inputready!(Pina0, "Pina0");

define_pin!(Pina1);
create_pin!(Pina1, porta, pin1);
read_write!(Pina1, "Pina1");
set_as!(Pina1, "Pina1");
outputready!(Pina1, "Pina1");
inputconfiguring!(Pina1, "Pina1");
inputready!(Pina1, "Pina1");

define_pin!(Pina2);
create_pin!(Pina2, porta, pin2);
read_write!(Pina2, "Pina2");
set_as!(Pina2, "Pina2");
outputready!(Pina2, "Pina2");
inputconfiguring!(Pina2, "Pina2");
inputready!(Pina2, "Pina2");

define_pin!(Pina3);
create_pin!(Pina3, porta, pin3);
read_write!(Pina3, "Pina3");
set_as!(Pina3, "Pina3");
outputready!(Pina3, "Pina3");
inputconfiguring!(Pina3, "Pina3");
inputready!(Pina3, "Pina3");

define_pin!(Pina4);
create_pin!(Pina4, porta, pin4);
read_write!(Pina4, "Pina4");
set_as!(Pina4, "Pina4");
outputready!(Pina4, "Pina4");
inputconfiguring!(Pina4, "Pina4");
inputready!(Pina4, "Pina4");

define_pin!(Pina5);
create_pin!(Pina5, porta, pin5);
read_write!(Pina5, "Pina5");
set_as!(Pina5, "Pina5");
outputready!(Pina5, "Pina5");
inputconfiguring!(Pina5, "Pina5");
inputready!(Pina5, "Pina5");

define_pin!(Pina6);
create_pin!(Pina6, porta, pin6);
read_write!(Pina6, "Pina6");
set_as!(Pina6, "Pina6");
outputready!(Pina6, "Pina6");
inputconfiguring!(Pina6, "Pina6");
inputready!(Pina6, "Pina6");

define_pin!(Pina7);
create_pin!(Pina7, porta, pin7);
read_write!(Pina7, "Pina7");
set_as!(Pina7, "Pina7");
outputready!(Pina7, "Pina7");
inputconfiguring!(Pina7, "Pina7");
inputready!(Pina7, "Pina7");

define_pin!(Pinb0);
create_pin!(Pinb0, portb, pin0);
read_write!(Pinb0, "Pinb0");
set_as!(Pinb0, "Pinb0");
outputready!(Pinb0, "Pinb0");
inputconfiguring!(Pinb0, "Pinb0");
inputready!(Pinb0, "Pinb0");

define_pin!(Pinb1);
create_pin!(Pinb1, portb, pin1);
read_write!(Pinb1, "Pinb1");
set_as!(Pinb1, "Pinb1");
outputready!(Pinb1, "Pinb1");
inputconfiguring!(Pinb1, "Pinb1");
inputready!(Pinb1, "Pinb1");

define_pin!(Pinb2);
create_pin!(Pinb2, portb, pin2);
read_write!(Pinb2, "Pinb2");
set_as!(Pinb2, "Pinb2");
outputready!(Pinb2, "Pinb2");
inputconfiguring!(Pinb2, "Pinb2");
inputready!(Pinb2, "Pinb2");

define_pin!(Pinb3);
create_pin!(Pinb3, portb, pin3);
read_write!(Pinb3, "Pinb3");
set_as!(Pinb3, "Pinb3");
outputready!(Pinb3, "Pinb3");
inputconfiguring!(Pinb3, "Pinb3");
inputready!(Pinb3, "Pinb3");

define_pin!(Pinb4);
create_pin!(Pinb4, portb, pin4);
read_write!(Pinb4, "Pinb4");
set_as!(Pinb4, "Pinb4");
outputready!(Pinb4, "Pinb4");
inputconfiguring!(Pinb4, "Pinb4");
inputready!(Pinb4, "Pinb4");

define_pin!(Pinb5);
create_pin!(Pinb5, portb, pin5);
read_write!(Pinb5, "Pinb5");
set_as!(Pinb5, "Pinb5");
outputready!(Pinb5, "Pinb5");
inputconfiguring!(Pinb5, "Pinb5");
inputready!(Pinb5, "Pinb5");

define_pin!(Pinb6);
create_pin!(Pinb6, portb, pin6);
read_write!(Pinb6, "Pinb6");
set_as!(Pinb6, "Pinb6");
outputready!(Pinb6, "Pinb6");
inputconfiguring!(Pinb6, "Pinb6");
inputready!(Pinb6, "Pinb6");

define_pin!(Pinb7);
create_pin!(Pinb7, portb, pin7);
read_write!(Pinb7, "Pinb7");
set_as!(Pinb7, "Pinb7");
outputready!(Pinb7, "Pinb7");
inputconfiguring!(Pinb7, "Pinb7");
inputready!(Pinb7, "Pinb7");

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
        let mut pina1: Pina1<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            Pina1::new(i2c.clone(), 0x40);
        let result = pina1.read_config(Register::Gpio);
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
        let mut pinb3: Pinb3<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            Pinb3::new(i2c.clone(), 0x40);
        let result = pinb3.read_config(Register::Gpio);
        assert_eq!(0xff, result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_pina() {
        let expectations = [
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpio as u8 | MyPort::Porta as u8, 0xff),
            ),
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpio as u8 | MyPort::Porta as u8, 0b11110111),
            ),
        ];
        let mut i2c = I2cMock::new(&expectations);
        let mut pina3: Pina3<embedded_hal_mock::common::Generic<I2cTransaction>, OutputReady> =
            Pina3 {
                i2c: i2c.clone(),
                address: 0x40,
                port: MyPort::Porta,
                pin: PinNumber::Pin3,
                state: core::marker::PhantomData::<OutputReady>,
            };
        let result = pina3.write(PinSet::High);
        assert_eq!((), result.unwrap());
        let result = pina3.write(PinSet::Low);
        assert_eq!((), result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_pinb() {
        let expectations = [
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Portb as u8),
                vector1(0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpio as u8 | MyPort::Portb as u8, 0xff),
            ),
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Portb as u8),
                vector1(0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpio as u8 | MyPort::Portb as u8, 0b11110111),
            ),
        ];
        let mut i2c = I2cMock::new(&expectations);
        let mut pinb3: Pinb3<embedded_hal_mock::common::Generic<I2cTransaction>, OutputReady> =
            Pinb3 {
                i2c: i2c.clone(),
                address: 0x40,
                port: MyPort::Portb,
                pin: PinNumber::Pin3,
                state: core::marker::PhantomData::<OutputReady>,
            };
        let result = pinb3.write(PinSet::High);
        assert_eq!((), result.unwrap());
        let result = pinb3.write(PinSet::Low);
        assert_eq!((), result.unwrap());

        //finalize execution
        i2c.done();
    }
}
// use crate::PinMask;

// mod sealed {
//     pub trait Sealed {}
// }

// pub trait PinState: sealed::Sealed {}
// pub trait OutputState: sealed::Sealed {}
// pub trait InputState: sealed::Sealed {
//     // ...
// }

// pub struct Output<S: OutputState> {
//     _p: core::marker::PhantomData<S>,
// }

// impl<S: OutputState> PinState for Output<S> {}
// impl<S: OutputState> sealed::Sealed for Output<S> {}

// pub struct OpenDrain;

// impl OutputState for OpenDrain {}
// impl sealed::Sealed for OpenDrain {}
// pub struct Input<S: InputState> {
//     _p: core::marker::PhantomData<S>,
// }

// impl<S: InputState> PinState for Input<S> {}
// impl<S: InputState> sealed::Sealed for Input<S> {}

// pub struct Floating;
// pub struct PullUp;

// impl InputState for Floating {}
// impl InputState for PullUp {}
// impl sealed::Sealed for Floating {}
// impl sealed::Sealed for PullUp {}

// pub struct PA1<S: PinState> {
//     mask: PinMask,
//     _p: core::marker::PhantomData<S>,
// }

// impl<S: PinState> PA1<S> {
//     pub fn into_input<N: InputState>(self, input: N) -> PA1<Input<N>> {
//         PA1 {
//             mask: PinMask::Pin1,
//             _p: core::marker::PhantomData::<Input<N>>,
//         }
//     }

//     pub fn into_output<N: OutputState>(self, output: N) -> PA1<Output<N>> {
//         PA1 {
//             mask: PinMask::Pin1,
//             _p: core::marker::PhantomData::<Output<N>>,
//         }
//     }
// }

// impl PA1<Input<PullUp>> {
//     pub fn read(&mut self) -> u8 {
//         8
//     }
// }

// impl PA1<Output<OpenDrain>> {
//     pub fn write(&mut self) {}
// }

// pub fn input_pull_up() -> PullUp {
//     PullUp
// }

// pub fn input_floating() -> Floating {
//     Floating
// }

// pub fn output_open_drain() -> OpenDrain {
//     OpenDrain
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::println;

//     #[test]
//     fn my_test() {
//         let pa = PA1::into_input(self, input_pull_up());
//         pa.read();

//         let pa1 = PA1::into_output(self, output_open_drain());
//         pa1.write();
//     }
// }
