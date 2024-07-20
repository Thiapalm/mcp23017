#![allow(unused)]

use crate::prelude::*;
use crate::registers::*;
use byteorder::{ByteOrder, LittleEndian};
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[derive(Debug, Clone, PartialEq)]
pub struct MCP23017<I2C, State = Configuring> {
    i2c: I2C,
    address: u8,
    state: core::marker::PhantomData<State>,
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), keep_self,),
    async(feature = "async", keep_self)
)]
trait RegReadWrite {
    async fn write_config(&mut self, register: Register, value: u16) -> Result<(), Error>;
    async fn read_config(&mut self, register: Register) -> Result<u16, Error>;
}

impl<I2C, E, State> MCP23017<I2C, State>
where
    I2C: I2c<Error = E>,
{
    /**
     * Function used to create a new handler for chip/port/pin
     */
    #[inline]
    pub fn new(i2c: I2C, address: u8) -> Self {
        MCP23017 {
            i2c,
            address,
            state: Default::default(),
        }
    }
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "MCP23017",),
    async(feature = "async", keep_self)
)]
impl<I2C, E, State> RegReadWrite for MCP23017<I2C, State>
where
    I2C: I2c<Error = E>,
{
    /**
     * Private function used to read the chip registers using i2c
     */
    #[inline]
    async fn read_config(&mut self, register: Register) -> Result<u16, Error> {
        let register_address = register as u8;
        let mut rx_buffer: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .await
            .map_err(i2c_comm_error)?;
        Ok(LittleEndian::read_u16(&rx_buffer))
    }

    /**
     * Private function used to write the chip registers using i2c
     */
    #[inline]
    async fn write_config(&mut self, register: Register, value: u16) -> Result<(), Error> {
        let register_address = register as u8;
        self.i2c
            .write(
                self.address,
                &[
                    register_address,
                    value.to_le_bytes()[0],
                    value.to_le_bytes()[1],
                ],
            )
            .await
            .map_err(i2c_comm_error)?;
        Ok(())
    }
}

#[allow(dead_code)]
#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "MCP23017",),
    async(feature = "async", keep_self)
)]
impl<I2C, E> MCP23017<I2C, Configuring>
where
    I2C: I2c<Error = E>,
{
    /**
     * Function used to set the chip/port/pin as input
     */
    #[inline]
    pub async fn set_as_input(mut self) -> Result<MCP23017<I2C, InputConfiguring>, Error> {
        self.write_config(Register::Iodir, 0xFFFF).await?;

        Ok(MCP23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<InputConfiguring>,
        })
    }

    /**
     * Function used to set the chip/port/pin as output
     */
    #[inline]
    pub async fn set_as_output(mut self) -> Result<MCP23017<I2C, OutputReady>, Error> {
        self.write_config(Register::Iodir, 0x0000).await?;

        Ok(MCP23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<OutputReady>,
        })
    }
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "MCP23017",),
    async(feature = "async", keep_self)
)]
impl<I2C, E> MCP23017<I2C, OutputReady>
where
    I2C: I2c<Error = E>,
{
    /**
     * Function used to write the output value to be set on chip/port/pin
     */
    #[inline]
    pub async fn write(&mut self, value: u16) -> Result<(), Error> {
        self.write_config(Register::Gpio, value)
            .await
            .map_err(i2c_comm_error)?;
        Ok(())
    }

    /**
     * Function used to write the output value to be set on pin
     */
    #[inline]
    pub async fn write_pin(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        value: PinSet,
    ) -> Result<(), Error> {
        let mut result = self.read_config(Register::Gpio).await?;

        let mut res = result.to_le_bytes();
        result = match (port, value) {
            (MyPort::Porta, PinSet::High) => {
                res[0] = bit_set(res[0], pin);
                LittleEndian::read_u16(&res)
            }
            (MyPort::Porta, PinSet::Low) => {
                res[0] = bit_clear(res[0], pin);
                LittleEndian::read_u16(&res)
            }
            (MyPort::Portb, PinSet::High) => {
                res[1] = bit_set(res[1], pin);
                LittleEndian::read_u16(&res)
            }
            (MyPort::Portb, PinSet::Low) => {
                res[1] = bit_clear(res[1], pin);
                LittleEndian::read_u16(&res)
            }
        };

        self.write_config(Register::Gpio, result)
            .await
            .map_err(i2c_comm_error)?;
        Ok(())
    }
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "MCP23017",),
    async(feature = "async", keep_self)
)]
impl<I2C, E> MCP23017<I2C, InputConfiguring>
where
    I2C: I2c<Error = E>,
{
    /**
     * Function used to set the pull on the input
     */
    #[inline]
    pub async fn set_pull(mut self, pull: PinSet) -> Result<Self, Error> {
        let result = match pull {
            PinSet::High => 0xFFFF,
            PinSet::Low => 0x0000,
        };

        self.write_config(Register::Gppu, result).await?;

        Ok(self)
    }

    /**
     * Function used to set the interrupt mirror function on the input
     */
    #[inline]
    pub async fn set_interrupt_mirror(mut self, mirror: InterruptMirror) -> Result<Self, Error> {
        let mut reg = self.read_config(Register::Iocon).await?;

        let mut regres = reg.to_le_bytes();
        match mirror {
            InterruptMirror::MirrorOn => {
                regres[0] |= InterruptMirror::MirrorOn as u8;
                regres[1] |= InterruptMirror::MirrorOn as u8;
            }
            InterruptMirror::MirrorOff => {
                regres[0] &= !(InterruptMirror::MirrorOn as u8);
                regres[1] &= !(InterruptMirror::MirrorOn as u8);
            }
        }
        reg = LittleEndian::read_u16(&regres);

        self.write_config(Register::Iocon, reg).await?;

        Ok(self)
    }

    /**
     * Function used to choose the pin as interrupt on the input
     */
    #[inline]
    pub async fn set_interrupt_on(
        mut self,
        port: MyPort,
        pin: PinNumber,
        interrupt_on: InterruptOn,
    ) -> Result<Self, Error> {
        let mut reg = self.read_config(Register::Intcon).await?;

        let mut regres = reg.to_le_bytes();
        reg = match (port, interrupt_on) {
            (MyPort::Porta, InterruptOn::PinChange) => {
                regres[0] = bit_clear(regres[0], pin);
                LittleEndian::read_u16(&regres)
            }
            (MyPort::Porta, InterruptOn::ChangeFromRegister) => {
                regres[0] = bit_set(regres[0], pin);
                LittleEndian::read_u16(&regres)
            }
            (MyPort::Portb, InterruptOn::PinChange) => {
                regres[1] = bit_clear(regres[1], pin);
                LittleEndian::read_u16(&regres)
            }
            (MyPort::Portb, InterruptOn::ChangeFromRegister) => {
                regres[1] = bit_set(regres[1], pin);
                LittleEndian::read_u16(&regres)
            }
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
        port: MyPort,
        pin: PinNumber,
        value: PinSet,
    ) -> Result<Self, Error> {
        let intcon = self.read_config(Register::Intcon).await?.to_le_bytes();

        match port {
            MyPort::Porta => {
                if bit_read(intcon[0], pin) != 1 {
                    return Err(Error::InvalidInterruptSetting);
                }
            }
            MyPort::Portb => {
                if bit_read(intcon[1], pin) != 1 {
                    return Err(Error::InvalidInterruptSetting);
                }
            }
        }

        let mut reg = self.read_config(Register::Defval).await?.to_le_bytes(); //change only valid if intcon is set to 1

        match (port, value) {
            (MyPort::Porta, PinSet::High) => {
                reg[0] = bit_set(reg[0], pin);
            }
            (MyPort::Porta, PinSet::Low) => {
                reg[0] = bit_clear(reg[0], pin);
            }
            (MyPort::Portb, PinSet::High) => {
                reg[1] = bit_set(reg[1], pin);
            }
            (MyPort::Portb, PinSet::Low) => {
                reg[1] = bit_clear(reg[1], pin);
            }
        };

        self.write_config(Register::Defval, LittleEndian::read_u16(&reg))
            .await?;
        Ok(self)
    }

    /**
     * Function used to set input to the ready state
     */
    #[inline]
    pub fn ready(mut self) -> MCP23017<I2C, InputReady> {
        MCP23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<InputReady>,
        }
    }
}

#[maybe_async_cfg::maybe(
    sync(cfg(not(feature = "async")), self = "MCP23017",),
    async(feature = "async", keep_self)
)]
impl<I2C, E> MCP23017<I2C, InputReady>
where
    I2C: I2c<Error = E>,
{
    /**
     * Function used to read the input
     */
    #[inline]
    pub async fn read(&mut self) -> Result<u16, Error> {
        let mut reg = self
            .read_config(Register::Gpio)
            .await
            .map_err(i2c_comm_error)?;
        Ok(reg)
    }

    /**
     * Function used to read the input pin
     */
    #[inline]
    pub async fn read_pin(&mut self, port: MyPort, pin: PinNumber) -> Result<u8, Error> {
        let mut result = self.read().await?.to_le_bytes();

        let result = match port {
            MyPort::Porta => bit_read(result[0], pin),
            MyPort::Portb => bit_read(result[1], pin),
        };

        Ok(result)
    }

    /**
     * Function used to disable the interrupt on the input
     */
    #[inline]
    pub async fn disable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
        let mut reg = self.read_config(Register::Gpinten).await?.to_le_bytes();

        match port {
            MyPort::Porta => reg[0] = bit_clear(reg[0], pin),
            MyPort::Portb => reg[1] = bit_clear(reg[1], pin),
        };
        let reg = LittleEndian::read_u16(&reg);

        self.write_config(Register::Gpinten, reg).await
    }

    /**
     * Function used to enable the interrupt on the input
     */
    #[inline]
    pub async fn enable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
        let mut reg = self.read_config(Register::Gpinten).await?.to_le_bytes();

        match port {
            MyPort::Porta => reg[0] = bit_set(reg[0], pin),
            MyPort::Portb => reg[1] = bit_set(reg[1], pin),
        };

        let reg = LittleEndian::read_u16(&reg);
        self.write_config(Register::Gpinten, reg).await
    }

    /**
     * Function used to verify the interrupt on the input
     */
    #[inline]
    pub async fn get_interrupted_pin(&mut self, port: MyPort) -> Option<PinNumber> {
        let pin_msk = self
            .read_config(Register::Intf)
            .await
            .unwrap_or(0)
            .to_le_bytes();

        let result = match port {
            MyPort::Porta => {
                if pin_msk[0] != 0 {
                    pin_msk[0]
                } else {
                    0
                }
            }
            MyPort::Portb => {
                if pin_msk[1] != 0 {
                    pin_msk[1]
                } else {
                    0
                }
            }
        };

        pin_mask_to_number(PinMask::from(result))
    }
}

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
    fn test_read_config_error() {
        let expectations =
            [
                I2cTransaction::write_read(
                    0x40,
                    vector1(Register::Gpio as u8),
                    vector2(0xff, 0xff),
                )
                .with_error(embedded_hal::i2c::ErrorKind::Other),
            ];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.read_config(Register::Gpio);
        assert_eq!(Error::CommunicationErr, result.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_config_success() {
        let expectations = [I2cTransaction::write_read(
            0x40,
            vector1(Register::Gpio as u8),
            vector2(0xad, 0xde),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.read_config(Register::Gpio);
        assert_eq!(0xdead, result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_config_error() {
        let expectations = [
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0xff, 0x10))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.write_config(Register::Gpio, 0x10ff);
        assert_eq!(Error::CommunicationErr, result.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_config_success() {
        let expectations = [I2cTransaction::write(
            0x40,
            vector3(Register::Gpio as u8, 0xff, 0x10),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.write_config(Register::Gpio, 0x10ff); //0xaabb
        assert_eq!((), result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_as_input_error() {
        let expectations =
            [
                I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff))
                    .with_error(embedded_hal::i2c::ErrorKind::Other),
            ];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_input();

        assert_eq!(Error::CommunicationErr, mcp.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_as_input_success() {
        let expectations = [I2cTransaction::write(
            0x40,
            vector3(Register::Iodir as u8, 0xff, 0xff),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_input().unwrap();

        assert_eq!(0x40, mcp.address);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_as_output_error() {
        let expectations =
            [
                I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0x00, 0x00))
                    .with_error(embedded_hal::i2c::ErrorKind::Other),
            ];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output();

        assert_eq!(Error::CommunicationErr, mcp.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_as_output_success() {
        let expectations = [I2cTransaction::write(
            0x40,
            vector3(Register::Iodir as u8, 0x00, 0x00),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output().unwrap();

        assert_eq!(0x40, mcp.address);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_success() {
        let expectations = [
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0x00, 0x00)),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0x11, 0x22)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output().unwrap();
        assert_eq!((), mcp.write(0x2211).unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_error() {
        let expectations = [
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0x00, 0x00)),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0x11, 0x22))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output().unwrap();
        assert_eq!(Error::CommunicationErr, mcp.write(0x2211).unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_pin_error() {
        let expectations = [
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0x00, 0x00)),
            I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector2(0xff, 0xff)),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0xff, 0xfe))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output().unwrap();

        let result = mcp.write_pin(MyPort::Portb, PinNumber::Pin0, PinSet::Low);
        assert_eq!(Error::CommunicationErr, result.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_pin_success() {
        let expectations = [
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0x00, 0x00)),
            I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector2(0xff, 0xff)),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0xff, 0xfe)),
            I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector2(0xff, 0xff)),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0xfe, 0xff)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output().unwrap();

        let result = mcp.write_pin(MyPort::Portb, PinNumber::Pin0, PinSet::Low);
        assert_eq!((), result.unwrap());
        let result = mcp.write_pin(MyPort::Porta, PinNumber::Pin0, PinSet::Low);
        assert_eq!((), result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_pull_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Gppu as u8, 0x00, 0x00)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp.set_as_input().unwrap().set_pull(PinSet::Low).unwrap();

        assert_eq!(0x40, result.address);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_pull_error() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Gppu as u8, 0x00, 0x00))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_pull(PinSet::Low)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_mirror_error() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_interrupt_mirror (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Iocon as u8), vector2(0xff, 0xff)),
            //set_interrupt_mirror (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iocon as u8, 0xbf, 0xbf))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_mirror(InterruptMirror::MirrorOff)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_mirror_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_interrupt_mirror (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Iocon as u8), vector2(0xff, 0xff)),
            //set_interrupt_mirror (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iocon as u8, 0xbf, 0xbf)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_mirror(InterruptMirror::MirrorOff)
            .unwrap();

        assert_eq!(0x40, result.address);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_on_error() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_interrupt_on (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Intcon as u8), vector2(0xff, 0xdd)),
            //set_interrupt_on (write_config)
            I2cTransaction::write(0x40, vector3(Register::Intcon as u8, 0xff, 0xdc))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_on(MyPort::Portb, PinNumber::Pin0, InterruptOn::PinChange)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_on_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_interrupt_on (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Intcon as u8), vector2(0xff, 0xdd)),
            //set_interrupt_on (write_config)
            I2cTransaction::write(0x40, vector3(Register::Intcon as u8, 0xff, 0xdc)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_on(MyPort::Portb, PinNumber::Pin0, InterruptOn::PinChange)
            .unwrap();

        assert_eq!(0x40, result.address);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_compare_error() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Intcon as u8), vector2(0xff, 0xff)),
            //set_interrupt_compare (write_config)
            I2cTransaction::write_read(0x40, vector1(Register::Defval as u8), vector2(0xff, 0xff)),
            I2cTransaction::write(0x40, vector3(Register::Defval as u8, 0xfe, 0xff))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_compare(MyPort::Porta, PinNumber::Pin0, PinSet::Low)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_compare_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Intcon as u8), vector2(0xff, 0xff)),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Defval as u8), vector2(0xff, 0xff)),
            //set_interrupt_compare (write_config)
            I2cTransaction::write(0x40, vector3(Register::Defval as u8, 0xfe, 0xff)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_compare(MyPort::Porta, PinNumber::Pin0, PinSet::Low)
            .unwrap();

        assert_eq!(0x40, result.address);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_ready_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(0x40, vector3(Register::Iodir as u8, 0xff, 0xff)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp.set_as_input().unwrap().ready();

        let compare = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        assert_eq!(compare.address, result.address);
        assert_eq!(compare.state, result.state);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_error() {
        let expectations = [
            //read
            I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector2(0xff, 0xff))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.read().unwrap_err();

        assert_eq!(Error::CommunicationErr, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_success() {
        let expectations = [
            //read
            I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector2(0xad, 0xde)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.read().unwrap();

        assert_eq!(0xdead, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_pin_error() {
        let expectations = [
            //read_pin
            I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector2(0xad, 0xde))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.read_pin(MyPort::Porta, PinNumber::Pin0).unwrap_err();

        assert_eq!(Error::CommunicationErr, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_pin_success() {
        let expectations = [
            //read_pin
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8),
                vector2(0x00, 0b00000001),
            ),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.read_pin(MyPort::Portb, PinNumber::Pin0).unwrap();

        assert_eq!(1, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_disable_interrupt_error() {
        let expectations = [
            //disable interrupt (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpinten as u8),
                vector2(0x00, 0b00000001),
            ),
            I2cTransaction::write(0x40, vector3(Register::Gpinten as u8, 0, 0))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp
            .disable_interrupt(MyPort::Portb, PinNumber::Pin0)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_disable_interrupt_success() {
        let expectations = [
            //disable interrupt (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpinten as u8),
                vector2(0x00, 0b00000001),
            ),
            I2cTransaction::write(0x40, vector3(Register::Gpinten as u8, 0, 0)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp
            .disable_interrupt(MyPort::Portb, PinNumber::Pin0)
            .unwrap();

        assert_eq!((), result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_enable_interrupt_error() {
        let expectations = [
            //enable_interrupt (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpinten as u8),
                vector2(0b00000000, 0b00000000),
            ),
            I2cTransaction::write(0x40, vector3(Register::Gpinten as u8, 1, 0))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp
            .enable_interrupt(MyPort::Porta, PinNumber::Pin0)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_enable_interrupt_success() {
        let expectations = [
            //enable_interrupt (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpinten as u8),
                vector2(0b00000000, 0b00000000),
            ),
            I2cTransaction::write(0x40, vector3(Register::Gpinten as u8, 1, 0)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp
            .enable_interrupt(MyPort::Porta, PinNumber::Pin0)
            .unwrap();

        assert_eq!((), result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_get_interrupted_pin_error() {
        let expectations = [
            //get_interrupted_pin (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Intf as u8),
                vector2(0x00, 0b11111111),
            )
            .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.get_interrupted_pin(MyPort::Porta);

        assert_eq!(None, result);
        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_get_interrupted_pin_success() {
        let expectations = [
            //get_interrupted_pin (read_config)
            I2cTransaction::write_read(0x40, vector1(Register::Intf as u8), vector2(0x00, 0x80)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.get_interrupted_pin(MyPort::Portb);

        assert_eq!(Some(PinNumber::Pin7), result);
        //finalize execution
        i2c.done();
    }
}
