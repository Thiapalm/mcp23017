#![allow(unused)]

use crate::interface::*;
use crate::registers;
use crate::registers::*;
use byteorder::{ByteOrder, LittleEndian};
#[cfg(not(feature = "async"))]
use embedded_hal::i2c::I2c;
#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c;

#[derive(Debug, Clone)]
pub struct MCP23017<I2C, State = Configuring> {
    i2c: I2C,
    address: u8,
    state: core::marker::PhantomData<State>,
}

// impl<I2C, E, State> Default for MCP23017<I2C, State>
// where
//     I2C: I2c<Error = E>,
// {
//     fn default() -> Self {
//         Self::new(None, registers::DEFAULT_ADDRESS)
//     }
// }

impl<I2C, E, State> MCP23017<I2C, State>
where
    I2C: I2c<Error = E>,
{
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
    #[inline]
    async fn read_config(&mut self, register: Register, port: MyPort) -> Result<u8, Error> {
        let register_address = register as u8 | port as u8;
        let mut rx_buffer: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .await
            .map_err(i2c_comm_error)?;
        Ok(rx_buffer[0])
    }

    #[inline]
    async fn write_config(
        &mut self,
        register: Register,
        port: MyPort,
        value: u8,
    ) -> Result<(), Error> {
        let register_address = register as u8 | port as u8;
        self.i2c
            .write(self.address, &[register_address, value])
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
    #[inline]
    pub async fn set_as_input(mut self) -> Result<MCP23017<I2C, InputConfiguring>, Error> {
        self.write_config(Register::Iodir, MyPort::Porta, 0xFF)
            .await?;
        self.write_config(Register::Iodir, MyPort::Portb, 0xFF)
            .await?;

        Ok(MCP23017 {
            i2c: self.i2c,
            address: self.address,
            state: core::marker::PhantomData::<InputConfiguring>,
        })
    }

    #[inline]
    pub async fn set_as_output(mut self) -> Result<MCP23017<I2C, OutputReady>, Error> {
        self.write_config(Register::Iodir, MyPort::Porta, 0x00)
            .await?;
        self.write_config(Register::Iodir, MyPort::Portb, 0x00)
            .await?;

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
    #[inline]
    pub async fn write(&mut self, value: u16) -> Result<(), Error> {
        let register_address = Register::Gpio as u8;
        let bytes = value.to_be_bytes();
        self.i2c
            .write(self.address, &[register_address, bytes[0], bytes[1]])
            .await
            .map_err(i2c_comm_error)?;
        Ok(())
    }

    #[inline]
    pub async fn write_pin(
        &mut self,
        port: MyPort,
        pin: PinNumber,
        value: PinSet,
    ) -> Result<(), Error> {
        let mut result = self.read_config(Register::Gpio, port).await?;

        result = match value {
            PinSet::High => bit_set(result, pin),
            PinSet::Low => bit_clear(result, pin),
        };

        let register_address = Register::Gpio as u8;

        self.i2c
            .write(self.address, &[register_address, result])
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
    #[inline]
    pub async fn set_pull(mut self, pull: PinSet) -> Result<Self, Error> {
        let result = match pull {
            PinSet::High => 0xFF,
            PinSet::Low => 0x00,
        };

        self.write_config(Register::Gppu, MyPort::Porta, result)
            .await?;
        self.write_config(Register::Gppu, MyPort::Portb, result)
            .await?;
        Ok(self)
    }

    #[inline]
    pub async fn set_interrupt_mirror(mut self, mirror: InterruptMirror) -> Result<Self, Error> {
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

        self.write_config(Register::Iocon, MyPort::Porta, rega)
            .await?;

        self.write_config(Register::Iocon, MyPort::Portb, regb)
            .await?;
        Ok(self)
    }

    #[inline]
    pub async fn set_interrupt_on(
        mut self,
        port: MyPort,
        pin: PinNumber,
        interrupt_on: InterruptOn,
    ) -> Result<Self, Error> {
        let mut reg = self.read_config(Register::Intcon, port).await?;

        reg = match interrupt_on {
            InterruptOn::PinChange => bit_clear(reg, pin),
            InterruptOn::ChangeFromRegister => bit_set(reg, pin),
        };

        self.write_config(Register::Intcon, port, reg).await?;
        Ok(self)
    }

    #[inline]
    pub async fn set_interrupt_compare(
        mut self,
        port: MyPort,
        pin: PinNumber,
        value: PinSet,
    ) -> Result<Self, Error> {
        let intcon = self.read_config(Register::Intcon, port)?;

        if bit_read(intcon, pin) != 1 {
            return Err(Error::InvalidInterruptSetting);
        }

        let mut reg = self.read_config(Register::Defval, port).await?; //change only valid if intcon is set to 1

        reg = match value {
            PinSet::High => bit_set(reg, pin),
            PinSet::Low => bit_clear(reg, pin),
        };

        self.write_config(Register::Defval, port, reg).await?;
        Ok(self)
    }

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
    #[inline]
    pub async fn read(&mut self) -> Result<u16, Error> {
        let register_address = Register::Gpio as u8;
        let mut rx_buffer: [u8; 2] = [0; 2];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .await
            .map_err(i2c_comm_error)?;
        Ok(LittleEndian::read_u16(&rx_buffer))
    }

    #[inline]
    pub async fn read_pin(&mut self, port: MyPort, pin: PinNumber) -> Result<u8, Error> {
        let register_address = Register::Gpio as u8 | port as u8;
        let mut rx_buffer: [u8; 1] = [0; 1];
        self.i2c
            .write_read(self.address, &[register_address], &mut rx_buffer)
            .await
            .map_err(i2c_comm_error)?;
        Ok(bit_read(rx_buffer[0], pin))
    }

    #[inline]
    pub async fn disable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
        let mut reg = self.read_config(Register::Gpinten, port)?;

        reg = bit_clear(reg, pin);

        self.write_config(Register::Gpinten, port, reg).await
    }

    #[inline]
    pub async fn enable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
        let mut reg = self.read_config(Register::Gpinten, port).await?;

        reg = bit_set(reg, pin);
        self.write_config(Register::Gpinten, port, reg).await
    }

    #[inline]
    pub async fn get_interrupted_pin(&mut self, port: MyPort) -> Option<PinNumber> {
        let pin_msk = self.read_config(Register::Intf, port).await.unwrap_or(0);

        pin_mask_to_number(PinMask::from(pin_msk))
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
                I2cTransaction::write_read(0x40, vector1(Register::Gpio as u8), vector1(0xff))
                    .with_error(embedded_hal::i2c::ErrorKind::Other),
            ];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.read_config(Register::Gpio, MyPort::Porta);
        assert_eq!(Error::CommunicationErr, result.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_read_config_success() {
        let expectations = [I2cTransaction::write_read(
            0x40,
            vector1(Register::Gpio as u8),
            vector1(0xff),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.read_config(Register::Gpio, MyPort::Porta);
        assert_eq!(0xff, result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_config_error() {
        let expectations = [
            I2cTransaction::write(0x40, vector2(Register::Gpio as u8, 0x10))
                .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.write_config(Register::Gpio, MyPort::Porta, 0x10);
        assert_eq!(Error::CommunicationErr, result.unwrap_err());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_write_config_success() {
        let expectations = [I2cTransaction::write(
            0x40,
            vector2(Register::Gpio as u8, 0x10),
        )];
        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);
        let result = mcp.write_config(Register::Gpio, MyPort::Porta, 0x10);
        assert_eq!((), result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_as_input_error() {
        let expectations = [
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            )
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
        let expectations = [
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
        ];
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
        let expectations = [
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0x00),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0x00),
            )
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
        let expectations = [
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0x00),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0x00),
            ),
        ];
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0x00),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0x00),
            ),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0x22, 0x11)),
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0x00),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0x00),
            ),
            I2cTransaction::write(0x40, vector3(Register::Gpio as u8, 0x22, 0x11))
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0x00),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0x00),
            ),
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Portb as u8),
                vector1(0xff),
            ),
            I2cTransaction::write(0x40, vector2(Register::Gpio as u8, 0xfe))
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0x00),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0x00),
            ),
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Portb as u8),
                vector1(0xff),
            ),
            I2cTransaction::write(0x40, vector2(Register::Gpio as u8, 0xfe)),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut mcp = mcp.set_as_output().unwrap();

        let result = mcp.write_pin(MyPort::Portb, PinNumber::Pin0, PinSet::Low);
        assert_eq!((), result.unwrap());

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_pull_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gppu as u8 | MyPort::Porta as u8, 0x00),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Gppu as u8 | MyPort::Portb as u8, 0x00),
            ),
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gppu as u8 | MyPort::Porta as u8, 0x00),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Gppu as u8 | MyPort::Portb as u8, 0x00),
            )
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            //set_interrupt_mirror (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Iocon as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_mirror (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Iocon as u8 | MyPort::Portb as u8),
                vector1(0xff),
            ),
            //set_interrupt_mirror (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iocon as u8 | MyPort::Porta as u8, 0xbf),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iocon as u8 | MyPort::Portb as u8, 0xbf),
            )
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            //set_interrupt_mirror (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Iocon as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_mirror (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Iocon as u8 | MyPort::Portb as u8),
                vector1(0xff),
            ),
            //set_interrupt_mirror (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iocon as u8 | MyPort::Porta as u8, 0xbf),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iocon as u8 | MyPort::Portb as u8, 0xbf),
            ),
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            //set_interrupt_on (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Intcon as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_on (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Intcon as u8 | MyPort::Porta as u8, 0xfe),
            )
            .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_on(MyPort::Porta, PinNumber::Pin0, InterruptOn::PinChange)
            .unwrap_err();

        assert_eq!(Error::CommunicationErr, result);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_on_success() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            //set_interrupt_on (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Intcon as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_on (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Intcon as u8 | MyPort::Porta as u8, 0xfe),
            ),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp: MCP23017<embedded_hal_mock::common::Generic<I2cTransaction>, Configuring> =
            MCP23017::new(i2c.clone(), 0x40);

        let mut result = mcp
            .set_as_input()
            .unwrap()
            .set_interrupt_on(MyPort::Porta, PinNumber::Pin0, InterruptOn::PinChange)
            .unwrap();

        assert_eq!(0x40, result.address);

        //finalize execution
        i2c.done();
    }

    #[test]
    fn test_set_interrupt_compare_error() {
        let expectations = [
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Intcon as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Defval as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_compare (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Defval as u8 | MyPort::Porta as u8, 0xfe),
            )
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Intcon as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_compare (read_config)
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Defval as u8 | MyPort::Porta as u8),
                vector1(0xff),
            ),
            //set_interrupt_compare (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Defval as u8 | MyPort::Porta as u8, 0xfe),
            ),
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
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Porta as u8, 0xff),
            ),
            //set_as_input (write_config)
            I2cTransaction::write(
                0x40,
                vector2(Register::Iodir as u8 | MyPort::Portb as u8, 0xff),
            ),
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
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Gpio as u8 | MyPort::Porta as u8),
                vector1(0xad),
            )
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
                vector1(Register::Gpio as u8 | MyPort::Porta as u8),
                vector1(0b00000001),
            ),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.read_pin(MyPort::Porta, PinNumber::Pin0).unwrap();

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
                vector1(Register::Gpinten as u8 | MyPort::Porta as u8),
                vector1(0b00000001),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpinten as u8 | MyPort::Porta as u8, 0),
            )
            .with_error(embedded_hal::i2c::ErrorKind::Other),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp
            .disable_interrupt(MyPort::Porta, PinNumber::Pin0)
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
                vector1(Register::Gpinten as u8 | MyPort::Porta as u8),
                vector1(0b00000001),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpinten as u8 | MyPort::Porta as u8, 0),
            ),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp
            .disable_interrupt(MyPort::Porta, PinNumber::Pin0)
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
                vector1(Register::Gpinten as u8 | MyPort::Porta as u8),
                vector1(0b00000000),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpinten as u8 | MyPort::Porta as u8, 1),
            )
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
                vector1(Register::Gpinten as u8 | MyPort::Porta as u8),
                vector1(0b00000000),
            ),
            I2cTransaction::write(
                0x40,
                vector2(Register::Gpinten as u8 | MyPort::Porta as u8, 1),
            ),
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
                vector1(Register::Intf as u8 | MyPort::Porta as u8),
                vector1(0b11111111),
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
            I2cTransaction::write_read(
                0x40,
                vector1(Register::Intf as u8 | MyPort::Porta as u8),
                vector1(0x80),
            ),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut mcp = MCP23017 {
            i2c: i2c.clone(),
            address: 0x40,
            state: core::marker::PhantomData::<InputReady>,
        };
        let result = mcp.get_interrupted_pin(MyPort::Porta);

        assert_eq!(Some(PinNumber::Pin7), result);
        //finalize execution
        i2c.done();
    }
}
