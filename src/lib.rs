#![no_std]

pub mod gpio;
pub mod spi;
pub mod ltdc;
pub mod rcc;

pub extern crate stm32f429 as pac;
pub extern crate embedded_hal;

pub trait Peripheral {
    fn enable_clock(&mut self);
    fn disable_clock(&mut self);
    fn reset(&mut self);
}

pub trait PeripheralRef {
    type Output;

    fn take() -> &'static mut Self::Output;
}

pub type InterruptHandler = fn();

#[cfg(test)]
mod tests {
    use embedded_hal::digital::OutputPin;

    use self::gpio::GPIOA;

    use super::*;

    #[test]
    fn it_works() {
        let gpioa = gpio::GPIOA::take();

        let mut led1 = gpioa.pin(0);
        let mut led2 = gpioa.pin(1);

        led1.set_high().unwrap();
        led2.set_low().unwrap();
    }
}
