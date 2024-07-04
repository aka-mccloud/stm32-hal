#![allow(dead_code)]

use core::{ fmt, ops::Deref, ptr::addr_of };

use crate::{ pac, rcc::RCC, Peripheral, PeripheralRef };

pub struct I2C(pac::i2c1::RegisterBlock);

impl Deref for I2C {
    type Target = pac::i2c1::RegisterBlock;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct I2C1;

impl PeripheralRef for I2C1 {
    type Output = I2C;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::I2C1::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct I2C2;

impl PeripheralRef for I2C2 {
    type Output = I2C;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::I2C2::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct I2C3;

impl PeripheralRef for I2C3 {
    type Output = I2C;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::I2C3::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

impl Peripheral for I2C {
    fn reset(&mut self) {
        let rcc = RCC::take();
        let ptr = addr_of!(self.0);

        match ptr {
            pac::I2C1::PTR => {
                rcc.apb1rstr().modify(|_, w| w.i2c1rst().set_bit());
                rcc.apb1rstr().modify(|_, w| w.i2c1rst().clear_bit());
            }
            pac::I2C2::PTR => {
                rcc.apb1rstr().modify(|_, w| w.i2c2rst().set_bit());
                rcc.apb1rstr().modify(|_, w| w.i2c2rst().clear_bit());
            }
            pac::I2C3::PTR => {
                rcc.apb1rstr().modify(|_, w| w.i2c3rst().set_bit());
                rcc.apb1rstr().modify(|_, w| w.i2c3rst().clear_bit());
            }
            _ => panic!(),
        }
    }

    fn enable_clock(&mut self) {
        let rcc = RCC::take();
        let ptr = addr_of!(self.0);

        match ptr {
            pac::I2C1::PTR => rcc.apb1enr().modify(|_, w| w.i2c1en().set_bit()),
            pac::I2C2::PTR => rcc.apb1enr().modify(|_, w| w.i2c2en().set_bit()),
            pac::I2C3::PTR => rcc.apb1enr().modify(|_, w| w.i2c3en().set_bit()),
            _ => panic!(),
        }
    }

    fn disable_clock(&mut self) {
        let rcc = RCC::take();
        let ptr = addr_of!(self.0);

        match ptr {
            pac::I2C1::PTR => rcc.apb1enr().modify(|_, w| w.i2c1en().clear_bit()),
            pac::I2C2::PTR => rcc.apb1enr().modify(|_, w| w.i2c2en().clear_bit()),
            pac::I2C3::PTR => rcc.apb1enr().modify(|_, w| w.i2c3en().clear_bit()),
            _ => panic!(),
        }
    }
}

impl I2C {
    pub fn enable(&mut self) {
        self.cr1().modify(|_, w| w.pe().set_bit())
    }

    pub fn disable(&mut self) {
        self.cr1().modify(|_, w| w.pe().clear_bit())
    }

    pub fn is_enabled(&self) -> bool {
        self.cr1().read().pe().bit()
    }

    pub fn init(&mut self, mode: I2CMode, speed_mode: SpeedMode, scl_freq: u32) -> Result<()> {
        self.disable();

        if scl_freq > 100_000 && speed_mode == SpeedMode::StandardMode {
            return Err(Error::InitError("Frequency is too high"));
        }

        if scl_freq <= 100_000 && speed_mode != SpeedMode::StandardMode {
            return Err(Error::InitError("Frequency is too low"));
        }

        // calculate CCR
        let f_pclk1 = RCC::take().pclk1_freq();

        let (ccr, trise) = match speed_mode {
            SpeedMode::StandardMode => (f_pclk1 / scl_freq / 2, f_pclk1 / 1_000_000 + 1),
            SpeedMode::FastModeDuty2 => (f_pclk1 / scl_freq / 3, (f_pclk1 * 3) / 10_000_000 + 1),
            SpeedMode::FastModeDuty16_9 =>
                (f_pclk1 / scl_freq / 25, (f_pclk1 * 3) / 10_000_000 + 1),
        };

        unsafe {
            self.cr1().modify(|_, w| w.ack().set_bit());
            self.cr2().modify(|_, w| w.freq().bits((f_pclk1 / 1_000_000) as _));
            match speed_mode {
                SpeedMode::StandardMode =>
                    self.ccr().modify(|_, w| {
                        w.f_s().clear_bit();
                        w.duty().clear_bit()
                    }),
                SpeedMode::FastModeDuty2 =>
                    self.ccr().modify(|_, w| {
                        w.f_s().set_bit();
                        w.duty().clear_bit()
                    }),
                SpeedMode::FastModeDuty16_9 =>
                    self.ccr().modify(|_, w| {
                        w.f_s().set_bit();
                        w.duty().set_bit()
                    }),
            }

            self.ccr().modify(|_, w| w.ccr().bits(ccr as _));
            self.trise().modify(|_, w| w.trise().set(trise as _));

            match mode {
                I2CMode::Master => (),
                I2CMode::Slave { addr1, addr2 } => {
                    self.oar1().write(|w| w.add().set(addr1 << 1));
                    if let Some(addr2) = addr2 {
                        self.oar2().write(|w| {
                            w.endual().set_bit();
                            w.add2().set(addr2)
                        });
                    }
                }
            }
        }

        self.enable();

        Ok(())
    }

    #[inline]
    pub fn is_busy(&self) -> bool {
        self.sr2().read().busy().bit_is_set()
    }

    #[inline]
    pub fn master_start(&mut self) -> Result<()> {
        while self.is_busy() {}

        // Generate START condition
        self.generate_start_condition_sync()
    }

    #[inline]
    pub fn master_stop(&mut self) -> Result<()> {
        // Send STOP condition
        self.generate_stop_condition_sync()?;

        Ok(())
    }

    #[inline]
    pub fn master_write_address(&mut self, addr: u8, read: bool) -> Result<()> {
        // Send ADDRESS
        self.master_send_address_sync(addr, read)?;

        // Clear ADDR flag
        self.sr1().read();
        self.sr2().read();

        Ok(())
    }

    #[inline]
    pub fn master_write_bytes(&mut self, data: &[u8]) -> Result<()> {
        // Send DATA
        for byte in data {
            self.master_write_byte_sync(*byte)?;
        }

        // Wait for transfer end
        while self.sr1().read().btf().bit_is_clear() {}

        Ok(())
    }

    #[inline]
    pub fn master_read_bytes(&mut self, data: &mut [u8]) -> Result<()> {
        if let Some((last, data)) = data.split_last_mut() {
            let mut ack = false;

            // Enable auto ACKing if receiving more than 1 byte
            self.cr1().modify(|r, w| {
                ack = r.ack().bit();
                w.ack().bit(!data.is_empty())
            });

            for byte in data {
                *byte = self.master_read_byte_sync()?;
            }

            self.cr1().modify(|_, w| w.ack().clear_bit());
            self.generate_stop_condition_sync()?;

            *last = self.master_read_byte_sync()?;

            self.cr1().modify(|_, w| w.ack().bit(ack));
        }

        Ok(())
    }

    pub fn master_write_data(&mut self, addr: u8, data: &[u8]) -> Result<()> {
        self.master_start()?;
        self.master_write_address(addr, false)?;
        self.master_write_bytes(data)?;
        self.master_stop()
    }

    pub fn master_read_data(&mut self, addr: u8, data: &mut [u8]) -> Result<()> {
        self.master_start()?;
        self.master_write_address(addr, true)?;
        self.master_read_bytes(data)
    }

    #[inline]
    fn generate_start_condition_async(&mut self) {
        self.cr1().modify(|_, w| w.start().set_bit())
    }

    #[inline]
    fn generate_start_condition_sync(&mut self) -> Result<()> {
        self.generate_start_condition_async();

        loop {
            let sr1 = self.sr1().read();
            if sr1.sb().bit_is_set() {
                return Ok(());
            }
            if sr1.berr().bit_is_set() {
                return Err(Error::BusError);
            }
        }
    }

    #[inline]
    fn generate_stop_condition_async(&mut self) {
        self.cr1().modify(|_, w| w.stop().set_bit());
    }

    #[inline]
    fn generate_stop_condition_sync(&mut self) -> Result<()> {
        Ok(self.generate_stop_condition_async())
    }

    #[inline]
    fn master_send_address_async(&mut self, addr: u8, read: bool) {
        self.dr().write(|w| w.dr().set(if read { (addr << 1) | 1u8 } else { (addr << 1) & !1u8 }));
    }

    #[inline]
    fn master_send_address_sync(&mut self, addr: u8, read: bool) -> Result<()> {
        self.master_send_address_async(addr, read);

        loop {
            let sr1 = self.sr1().read();
            if sr1.addr().bit_is_clear() && sr1.add10().bit_is_clear() {
                self.check_for_errors()?;
            } else {
                return Ok(());
            }
        }
    }

    #[inline]
    fn master_write_byte_async(&mut self, byte: u8) {
        self.dr().write(|w| w.dr().set(byte))
    }

    #[inline]
    fn master_write_byte_sync(&mut self, byte: u8) -> Result<()> {
        self.master_write_byte_async(byte);

        while self.sr1().read().tx_e().bit_is_clear() {
            self.check_for_errors()?;
        }

        Ok(())
    }

    #[inline]
    pub fn master_read_byte_async(&mut self) -> u8 {
        self.dr().read().dr().bits()
    }

    #[inline]
    pub fn master_read_byte_sync(&mut self) -> Result<u8> {
        while self.sr1().read().rx_ne().bit_is_clear() {
            self.check_for_errors()?;
        }

        Ok(self.master_read_byte_async())
    }

    fn check_for_errors(&self) -> Result<()> {
        let sr1 = self.sr1().read();

        if sr1.af().bit_is_set() {
            Err(Error::AcknowledgeFailure)
        } else if sr1.arlo().bit_is_set() {
            Err(Error::ArbitrationLost)
        } else if sr1.berr().bit_is_set() {
            Err(Error::BusError)
        } else {
            Ok(())
        }
    }
}

pub struct I2CConfig {
    pub mode: I2CMode,
    pub speed_mode: SpeedMode,
    pub scl_freq: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusMode {
    I2C = 0b0,
    SMBus = 0b1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SMBusType {
    Device = 0b0,
    Host = 0b1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedMode {
    StandardMode = 0b00,
    FastModeDuty2 = 0b10,
    FastModeDuty16_9 = 0b11,
}

#[derive(Debug)]
pub enum I2CMode {
    Master,
    Slave {
        addr1: u16,
        addr2: Option<u8>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InitError(&'static str),
    BusError,
    ArbitrationLost,
    AcknowledgeFailure,
    BusyError(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InitError(e) => f.write_fmt(format_args!("InitError: {}", e)),
            Error::BusError => f.write_str("Bus Error"),
            Error::ArbitrationLost => f.write_str("Arbitration Lost"),
            Error::AcknowledgeFailure => f.write_str("Acknowledge Failure"),
            Error::BusyError(e) => f.write_fmt(format_args!("BusyError: {}", e)),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
