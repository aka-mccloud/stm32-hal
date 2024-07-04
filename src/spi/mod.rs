use core::{ fmt, mem, ptr::addr_of, slice };

use crate::{ pac, rcc::RCC, Peripheral, PeripheralRef };

pub struct SPI(pac::spi1::RegisterBlock);

pub struct SPI1;

impl PeripheralRef for SPI1 {
    type Output = SPI;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::SPI1::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct SPI2;

impl PeripheralRef for SPI2 {
    type Output = SPI;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::SPI2::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct SPI3;

impl PeripheralRef for SPI3 {
    type Output = SPI;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::SPI3::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct SPI4;

impl PeripheralRef for SPI4 {
    type Output = SPI;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::SPI4::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct SPI5;

impl PeripheralRef for SPI5 {
    type Output = SPI;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::SPI5::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

pub struct SPI6;

impl PeripheralRef for SPI6 {
    type Output = SPI;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::SPI6::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

impl Peripheral for SPI {
    fn enable_clock(&mut self) {
        let rcc = RCC::take();
        let ptr = addr_of!(self.0);

        match ptr {
            pac::SPI1::PTR => rcc.apb2enr().modify(|_, w| w.spi1en().set_bit()),
            pac::SPI2::PTR => rcc.apb1enr().modify(|_, w| w.spi2en().set_bit()),
            pac::SPI3::PTR => rcc.apb1enr().modify(|_, w| w.spi3en().set_bit()),
            pac::SPI4::PTR => rcc.apb2enr().modify(|_, w| w.spi4en().set_bit()),
            pac::SPI5::PTR => rcc.apb2enr().modify(|_, w| w.spi5en().set_bit()),
            pac::SPI6::PTR => rcc.apb2enr().modify(|_, w| w.spi6en().set_bit()),
            _ => panic!(),
        }
    }

    fn disable_clock(&mut self) {
        let rcc = RCC::take();
        let ptr = addr_of!(self.0);

        match ptr {
            pac::SPI1::PTR => rcc.apb2enr().modify(|_, w| w.spi1en().clear_bit()),
            pac::SPI2::PTR => rcc.apb1enr().modify(|_, w| w.spi2en().clear_bit()),
            pac::SPI3::PTR => rcc.apb1enr().modify(|_, w| w.spi3en().clear_bit()),
            pac::SPI4::PTR => rcc.apb2enr().modify(|_, w| w.spi4en().clear_bit()),
            pac::SPI5::PTR => rcc.apb2enr().modify(|_, w| w.spi5en().clear_bit()),
            pac::SPI6::PTR => rcc.apb2enr().modify(|_, w| w.spi6en().clear_bit()),
            _ => panic!(),
        }
    }

    fn reset(&mut self) {
        let rcc = RCC::take();
        let ptr = addr_of!(self.0);

        match ptr {
            pac::SPI1::PTR => {
                rcc.apb2rstr().modify(|_, w| w.spi1rst().set_bit());
                rcc.apb2rstr().modify(|_, w| w.spi1rst().clear_bit());
            }
            pac::SPI2::PTR => {
                rcc.apb1rstr().modify(|_, w| w.spi2rst().set_bit());
                rcc.apb1rstr().modify(|_, w| w.spi2rst().clear_bit());
            }
            pac::SPI3::PTR => {
                rcc.apb1rstr().modify(|_, w| w.spi3rst().set_bit());
                rcc.apb1rstr().modify(|_, w| w.spi3rst().clear_bit());
            }
            pac::SPI4::PTR => {
                rcc.apb2rstr().modify(|_, w| w.spi4rst().set_bit());
                rcc.apb2rstr().modify(|_, w| w.spi4rst().clear_bit());
            }
            pac::SPI5::PTR => {
                rcc.apb2rstr().modify(|_, w| w.spi5rst().set_bit());
                rcc.apb2rstr().modify(|_, w| w.spi5rst().clear_bit());
            }
            pac::SPI6::PTR => {
                rcc.apb2rstr().modify(|_, w| w.spi6rst().set_bit());
                rcc.apb2rstr().modify(|_, w| w.spi6rst().clear_bit());
            }
            _ => panic!(),
        }
    }
}

impl SPI {
    pub fn enable(&mut self) {
        self.0.cr1().modify(|_, w| w.spe().set_bit());
    }

    pub fn disable(&mut self) {
        self.0.cr1().modify(|_, w| w.spe().clear_bit());
    }

    pub fn is_enabled(&self) -> bool {
        self.0.cr1().read().spe().bit()
    }

    pub fn init(&mut self, config: SPIConfig) -> Result<()> {
        self.enable_clock();
        self.reset();

        let SPIConfig { mode, bus_config, baud_rate, data_format, cpol, cpha, ssm } = config;

        self.0.cr1().modify(|_, w| {
            w.mstr().bit(mode == Mode::Master);
            w.ssi().bit(mode == Mode::Master);
            match bus_config {
                BusConfiguration::FullDuplex => {
                    w.bidimode().clear_bit();
                }
                BusConfiguration::HalfDuplex => {
                    w.bidimode().set_bit();
                }
                BusConfiguration::SimplexReceiveOnly => {
                    w.bidimode().clear_bit();
                    w.rxonly().set_bit();
                }
            }
            w.br().set(baud_rate as u8);
            w.dff().bit(data_format == DataFrameFormat::Format16Bit);
            w.cpol().bit(cpol == ClockPolarity::IdleHigh);
            w.cpha().bit(cpha == ClockPhase::SecondClockTransition);
            w.ssm().bit(ssm)
        });
        self.0.cr2().modify(|_, w| w.ssoe().bit(ssm));

        self.enable();

        Ok(())
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<()> {
        let dff = DataFrameFormat::from_bits(self.0.cr1().read().dff().bit() as u32);

        unsafe {
            match dff {
                DataFrameFormat::Format8Bit => {
                    for byte in data {
                        self.0.dr().write(|w| w.dr().set(*byte as _));
                        while self.0.sr().read().txe().bit_is_clear() {}
                    }
                }
                DataFrameFormat::Format16Bit => {
                    let data = slice::from_raw_parts(
                        data.as_ptr() as *const _,
                        data.len() / mem::size_of::<u16>()
                    );
                    for word in data {
                        self.0.dr().write(|w| w.dr().set(*word));
                        while self.0.sr().read().txe().bit_is_clear() {}
                    }
                }
            }

            // wait for busy flag is reset
            while self.0.sr().read().bsy().bit_is_set() {}

            let _ = self.0.dr().read().bits();
            _ = self.0.sr().read().bits();
        }

        Ok(())
    }

    pub fn read_data(&mut self, _data: &mut [u8]) -> Result<()> {
        Ok(())
    }
}

pub struct SPIConfig {
    pub mode: Mode,
    pub bus_config: BusConfiguration,
    pub baud_rate: BaudRate,
    pub data_format: DataFrameFormat,
    pub cpol: ClockPolarity,
    pub cpha: ClockPhase,
    pub ssm: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Slave = 0b0,
    Master = 0b1,
}

impl Mode {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::Slave,
            0b1 => Self::Master,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockPhase {
    FirstClockTransition = 0b0,
    SecondClockTransition = 0b1,
}

impl ClockPhase {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::FirstClockTransition,
            0b1 => Self::SecondClockTransition,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockPolarity {
    IdleLow = 0b0,
    IdleHigh = 0b1,
}

impl ClockPolarity {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::IdleLow,
            0b1 => Self::IdleHigh,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaudRate {
    FpclkDiv2 = 0b000,
    FpclkDiv4 = 0b001,
    FpclkDiv8 = 0b010,
    FpclkDiv16 = 0b011,
    FpclkDiv32 = 0b100,
    FpclkDiv64 = 0b101,
    FpclkDiv128 = 0b110,
    FpclkDiv256 = 0b111,
}

impl BaudRate {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b000 => Self::FpclkDiv2,
            0b001 => Self::FpclkDiv4,
            0b010 => Self::FpclkDiv8,
            0b011 => Self::FpclkDiv16,
            0b100 => Self::FpclkDiv32,
            0b101 => Self::FpclkDiv64,
            0b110 => Self::FpclkDiv128,
            0b111 => Self::FpclkDiv256,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    MSBFirst = 0b0,
    LSBFirst = 0b1,
}

impl FrameFormat {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::MSBFirst,
            0b1 => Self::LSBFirst,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormatMode {
    Motorola = 0b0,
    TI = 0b1,
}

impl FrameFormatMode {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::Motorola,
            0b1 => Self::TI,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFrameFormat {
    Format8Bit = 0b0,
    Format16Bit = 0b1,
}

impl DataFrameFormat {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::Format8Bit,
            0b1 => Self::Format16Bit,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BidirectionalMode {
    Receive = 0b0,
    Transmit = 0b1,
}

impl BidirectionalMode {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::Receive,
            0b1 => Self::Transmit,
            _ => panic!(),
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        val as _
    }
}

pub enum BusConfiguration {
    FullDuplex,
    HalfDuplex,
    SimplexReceiveOnly,
}

#[derive(Debug)]
pub enum Error {
    InitError(&'static str),
    BusError(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InitError(e) => f.write_fmt(format_args!("InitError: {}", e)),
            Error::BusError(e) => f.write_fmt(format_args!("BusError: {}", e)),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
