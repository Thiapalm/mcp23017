pub mod chipmode {

    #[derive(Debug, Clone)]
    pub struct Mcp23017<I2C, State = Configuring> {
        i2c: I2C,
        address: u8,
        state: core::marker::PhantomData<State>,
    }

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

    impl<I2C, E, State> RegReadWrite for Mcp23017<I2C, State>
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

        fn write_config(
            &mut self,
            register: Register,
            port: MyPort,
            value: u8,
        ) -> Result<(), Error> {
            let register_address = register as u8 | port as u8;
            self.i2c
                .write(self.address, &[register_address, value])
                .map_err(i2c_comm_error)?;
            Ok(())
        }
    }

    #[allow(dead_code)]
    impl<I2C, E> Mcp23017<I2C, Configuring>
    where
        I2C: i2c::I2c<Error = E>,
    {
        pub fn set_as_input(mut self) -> Result<Mcp23017<I2C, InputConfiguring>, Error> {
            self.write_config(Register::Iodir, MyPort::Porta, 0xFF)?;
            self.write_config(Register::Iodir, MyPort::Portb, 0xFF)?;

            Ok(Mcp23017 {
                i2c: self.i2c,
                address: self.address,
                state: core::marker::PhantomData::<InputConfiguring>,
            })
        }

        pub fn set_as_output(mut self) -> Result<Mcp23017<I2C, OutputReady>, Error> {
            self.write_config(Register::Iodir, MyPort::Porta, 0x00)?;
            self.write_config(Register::Iodir, MyPort::Portb, 0x00)?;

            Ok(Mcp23017 {
                i2c: self.i2c,
                address: self.address,
                state: core::marker::PhantomData::<OutputReady>,
            })
        }
    }

    impl<I2C, E> Mcp23017<I2C, OutputReady>
    where
        I2C: i2c::I2c<Error = E>,
    {
        pub fn write(&mut self, value: u16) -> Result<(), Error> {
            let register_address = Register::Gpio as u8;
            let bytes = value.to_be_bytes();
            self.i2c
                .write(self.address, &[register_address, bytes[0], bytes[1]])
                .map_err(i2c_comm_error)?;
            Ok(())
        }

        pub fn write_pin(
            &mut self,
            port: MyPort,
            pin: PinNumber,
            value: PinSet,
        ) -> Result<(), Error> {
            let mut result = self.read_config(Register::Gpio, port)?;

            result = match value {
                PinSet::High => bit_set(result, pin),
                PinSet::Low => bit_clear(result, pin),
            };

            let register_address = Register::Gpio as u8;

            self.i2c
                .write(self.address, &[register_address, result])
                .map_err(i2c_comm_error)?;
            Ok(())
        }
    }

    impl<I2C, E> Mcp23017<I2C, InputConfiguring>
    where
        I2C: i2c::I2c<Error = E>,
    {
        pub fn set_pull(&mut self, pull: PinSet) -> Result<&mut Self, Error> {
            let result = match pull {
                PinSet::High => 0xFF,
                PinSet::Low => 0x00,
            };

            self.write_config(Register::Gppu, MyPort::Porta, result)?;
            self.write_config(Register::Gppu, MyPort::Portb, result)?;
            Ok(self)
        }

        pub fn set_interrupt_mirror(
            &mut self,
            mirror: InterruptMirror,
        ) -> Result<&mut Self, Error> {
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

            self.write_config(Register::Iocon, MyPort::Portb, regb)?;
            Ok(self)
        }

        pub fn set_interrupt_on(
            &mut self,
            port: MyPort,
            pin: PinNumber,
            interrupt_on: InterruptOn,
        ) -> Result<&mut Self, Error> {
            let mut reg = self.read_config(Register::Intcon, port)?;

            reg = match interrupt_on {
                InterruptOn::PinChange => bit_clear(reg, pin),
                InterruptOn::ChangeFromRegister => bit_set(reg, pin),
            };

            self.write_config(Register::Intcon, port, reg)?;
            Ok(self)
        }

        pub fn set_interrupt_compare(
            &mut self,
            port: MyPort,
            pin: PinNumber,
            value: PinSet,
        ) -> Result<&mut Self, Error> {
            let intcon = self.read_config(Register::Intcon, port)?;

            if bit_read(intcon, pin) != 1 {
                return Err(Error::InvalidInterruptSetting);
            }

            let mut reg = self.read_config(Register::Defval, port)?; //change only valid if intcon is set to 1

            reg = match value {
                PinSet::High => bit_set(reg, pin),
                PinSet::Low => bit_clear(reg, pin),
            };

            self.write_config(Register::Defval, port, reg)?;
            Ok(self)
        }

        pub fn ready(self) -> Mcp23017<I2C, InputReady> {
            Mcp23017 {
                i2c: self.i2c,
                address: self.address,
                state: core::marker::PhantomData::<InputReady>,
            }
        }
    }

    impl<I2C, E> Mcp23017<I2C, InputReady>
    where
        I2C: i2c::I2c<Error = E>,
    {
        pub fn read(&mut self) -> Result<u16, Error> {
            let register_address = Register::Gpio as u8;
            let mut rx_buffer: [u8; 2] = [0; 2];
            self.i2c
                .write_read(self.address, &[register_address], &mut rx_buffer)
                .map_err(i2c_comm_error)?;
            Ok(LittleEndian::read_u16(&rx_buffer))
        }

        pub fn read_pin(&mut self, port: MyPort, pin: PinNumber) -> Result<u8, Error> {
            let register_address = Register::Gpio as u8 | port as u8;
            let mut rx_buffer: [u8; 1] = [0; 1];
            self.i2c
                .write_read(self.address, &[register_address], &mut rx_buffer)
                .map_err(i2c_comm_error)?;
            Ok(bit_read(rx_buffer[0], pin))
        }

        pub fn disable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
            let mut reg = self.read_config(Register::Gpinten, port)?;

            reg = bit_clear(reg, pin);

            self.write_config(Register::Gpinten, port, reg)
        }

        pub fn enable_interrupt(&mut self, port: MyPort, pin: PinNumber) -> Result<(), Error> {
            let mut reg = self.read_config(Register::Gpinten, port)?;

            reg = bit_set(reg, pin);
            self.write_config(Register::Gpinten, port, reg)
        }

        pub fn get_interrupted_pin(&mut self, port: MyPort) -> Option<PinNumber> {
            let pin_msk = self.read_config(Register::Intf, port).unwrap();

            pin_mask_to_number(PinMask::from(pin_msk))
        }
    }
}
