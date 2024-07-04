use core::convert::Infallible;

use super::Port;

use embedded_hal::digital::{ ErrorType, InputPin, OutputPin, StatefulOutputPin };

pub struct Pin {
    pub(crate) port: &'static Port,
    pub(crate) pin: u8,
}

impl ErrorType for Pin {
    type Error = Infallible;
}

impl InputPin for Pin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok((self.port.idr().read().bits() & (1 << self.pin)) != 0)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok((self.port.idr().read().bits() & (1 << self.pin)) == 0)
    }
}

impl OutputPin for Pin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.port.bsrr().write(|w| w.bits(1 << (self.pin + 16)));
        }

        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        unsafe {
            self.port.bsrr().write(|w| w.bits(1 << self.pin));
        }

        Ok(())
    }
}

impl StatefulOutputPin for Pin {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        Ok((self.port.odr().read().bits() & (1 << self.pin)) != 0)
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        Ok((self.port.odr().read().bits() & (1 << self.pin)) == 0)
    }
}

impl Pin {
    #[inline]
    pub fn set_mode(&mut self, mode: PinMode) {
        const MASK: u32 = 0b11;

        unsafe {
            let pos = self.pin * 2;
            self.port.moder().modify(|r, w| {
                w.bits((r.bits() & !(MASK << pos)) | ((mode as u32) << pos))
            });
        }
    }

    #[inline]
    pub fn set_output_type(&mut self, output_type: OutputType) {
        const MASK: u32 = 0b1;

        unsafe {
            let pos = self.pin;
            self.port.otyper().modify(|r, w| {
                match output_type {
                    OutputType::PushPull => w.bits(r.bits() & !(MASK << pos)),
                    OutputType::OpenDrain => w.bits(r.bits() | (MASK << pos)),
                }
            });
        }
    }

    #[inline]
    pub fn set_speed(&mut self, speed: Speed) {
        const MASK: u32 = 0b11;

        unsafe {
            let pos = self.pin * 2;
            self.port.ospeedr().modify(|r, w| {
                w.bits((r.bits() & !(MASK << pos)) | ((speed as u32) << pos))
            });
        }
    }

    #[inline]
    pub fn set_pull(&mut self, pull: Pull) {
        const MASK: u32 = 0b11;

        unsafe {
            let pos = self.pin * 2;
            self.port.pupdr().modify(|r, w| {
                w.bits((r.bits() & !(MASK << pos)) | ((pull as u32) << pos))
            });
        }
    }

    #[inline]
    pub fn set_alternate_function(&mut self, func: u8) {
        const MASK: u32 = 0b1111;

        unsafe {
            if self.pin < 8 {
                let pos = self.pin * 4;
                self.port.afrl().modify(|r, w| {
                    w.bits((r.bits() & !(MASK << pos)) | ((func as u32) << pos))
                });
            } else {
                let pos = (self.pin % 8) * 4;
                self.port.afrh().modify(|r, w| {
                    w.bits((r.bits() & !(MASK << pos)) | ((func as u32) << pos))
                });
            }
        }
    }
}

// pub type InterruptHandler = fn();

// fn default_handler() {}

// static mut IRQ_HANDLERS: [InterruptHandler; 16] = [default_handler; 16];

pub struct Input {
    pin: Pin,
}

impl ErrorType for Input {
    type Error = <Pin as ErrorType>::Error;
}

impl InputPin for Input {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }
}

impl Input {
    #[inline]
    pub fn new(pin: Pin, pull: Pull) -> Self {
        let mut input = Self { pin };

        input.pin.set_mode(PinMode::Input);
        input.pin.set_pull(pull);
        input
    }
}

impl Drop for Input {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_mode(PinMode::Analog);
        self.pin.set_output_type(OutputType::PushPull);
        self.pin.set_speed(Speed::Low);
        self.pin.set_pull(Pull::None);
        self.pin.set_alternate_function(0);
    }
}

pub struct Output {
    pin: Pin,
}

impl ErrorType for Output {
    type Error = <Pin as ErrorType>::Error;
}

impl OutputPin for Output {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high()
    }
}

impl StatefulOutputPin for Output {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_set_high()
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_set_low()
    }
}

impl Output {
    #[inline]
    pub fn new(pin: Pin, speed: Speed) -> Self {
        let mut output = Self { pin };

        output.pin.set_mode(PinMode::Output);
        output.pin.set_output_type(OutputType::PushPull);
        output.pin.set_speed(speed);
        output.pin.set_pull(Pull::None);
        output
    }
}

impl Drop for Output {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_mode(PinMode::Analog);
        self.pin.set_output_type(OutputType::PushPull);
        self.pin.set_speed(Speed::Low);
        self.pin.set_pull(Pull::None);
        self.pin.set_alternate_function(0);
    }
}

pub struct OpenDrain {
    pin: Pin,
}

impl ErrorType for OpenDrain {
    type Error = <Pin as ErrorType>::Error;
}

impl OutputPin for OpenDrain {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low()
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high()
    }
}

impl StatefulOutputPin for OpenDrain {
    fn is_set_high(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_set_high()
    }

    fn is_set_low(&mut self) -> Result<bool, Self::Error> {
        self.pin.is_set_low()
    }
}

impl OpenDrain {
    #[inline]
    pub fn new(pin: Pin, speed: Speed, pull: Pull) -> Self {
        let mut od = Self { pin };

        od.pin.set_mode(PinMode::Output);
        od.pin.set_output_type(OutputType::OpenDrain);
        od.pin.set_speed(speed);
        od.pin.set_pull(pull);
        od
    }
}

impl Drop for OpenDrain {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_mode(PinMode::Analog);
        self.pin.set_output_type(OutputType::PushPull);
        self.pin.set_speed(Speed::Low);
        self.pin.set_pull(Pull::None);
        self.pin.set_alternate_function(0);
    }
}

pub struct Alternate {
    pin: Pin,
}

impl Alternate {
    #[inline]
    pub fn new(pin: Pin, otype: OutputType, speed: Speed, pull: Pull, func: u8) -> Self {
        let mut af = Self { pin };

        af.pin.set_mode(PinMode::Alternate);
        af.pin.set_output_type(otype);
        af.pin.set_speed(speed);
        af.pin.set_pull(pull);
        af.pin.set_alternate_function(func);
        af
    }
}

impl Drop for Alternate {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_mode(PinMode::Analog);
        self.pin.set_output_type(OutputType::PushPull);
        self.pin.set_speed(Speed::Low);
        self.pin.set_pull(Pull::None);
        self.pin.set_alternate_function(0);
    }
}

pub struct Analog {
    pin: Pin,
}

impl Analog {
    #[inline]
    pub fn new(pin: Pin) -> Self {
        let mut analog = Self { pin };

        analog.pin.set_mode(PinMode::Analog);
        analog.pin.set_pull(Pull::None);
        analog
    }
}

impl Drop for Analog {
    #[inline]
    fn drop(&mut self) {
        self.pin.set_mode(PinMode::Analog);
        self.pin.set_output_type(OutputType::PushPull);
        self.pin.set_speed(Speed::Low);
        self.pin.set_pull(Pull::None);
        self.pin.set_alternate_function(0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    Input = 0b00,
    Output = 0b01,
    Alternate = 0b10,
    Analog = 0b11,
}

impl From<u32> for PinMode {
    fn from(value: u32) -> Self {
        match value {
            0b00 => Self::Input,
            0b01 => Self::Output,
            0b10 => Self::Alternate,
            0b11 => Self::Analog,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    PushPull = 0b0,
    OpenDrain = 0b1,
}

impl OutputType {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b0 => Self::PushPull,
            0b1 => Self::OpenDrain,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Low = 0b00,
    Medium = 0b01,
    High = 0b10,
    VeryHigh = 0b11,
}

impl Speed {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b00 => Self::Low,
            0b01 => Self::Medium,
            0b10 => Self::High,
            0b11 => Self::VeryHigh,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pull {
    None = 0b00,
    Up = 0b01,
    Down = 0b10,
}

impl Pull {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b00 => Self::None,
            0b01 => Self::Up,
            0b10 => Self::Down,
            _ => panic!(),
        }
    }
}

pub enum PinConfig {
    Input(Pull),
    Output(Speed),
    OpenDrain(Speed, Pull),
    Alternate(OutputType, Speed, Pull, u8),
    Analog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Low = 0b0,
    High = 0b1,
}

impl Level {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b0 => Self::Low,
            0b1 => Self::High,
            _ => panic!(),
        }
    }
}

impl From<Level> for bool {
    fn from(value: Level) -> Self {
        match value {
            Level::Low => false,
            Level::High => true,
        }
    }
}

impl From<bool> for Level {
    fn from(value: bool) -> Self {
        match value {
            true => Level::High,
            false => Level::Low,
        }
    }
}
