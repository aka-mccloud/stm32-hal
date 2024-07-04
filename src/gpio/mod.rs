pub mod pin;

use core::{ops::{ BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, Not }, ptr::addr_of};

use crate::{ pac, rcc::RCC, Peripheral, PeripheralRef };

use self::pin::{ OutputType, Pin, PinConfig, PinMode, Pull, Speed };

pub struct Port(pac::gpioa::RegisterBlock);

impl Deref for Port {
    type Target = pac::gpioa::RegisterBlock;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct GPIOA;

impl PeripheralRef for GPIOA {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOA::PTR as *mut _) }
    }
}

pub struct GPIOB;

impl PeripheralRef for GPIOB {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOB::PTR as *mut _) }
    }
}

pub struct GPIOC;

impl PeripheralRef for GPIOC {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOC::PTR as *mut _) }
    }
}

pub struct GPIOD;

impl PeripheralRef for GPIOD {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOD::PTR as *mut _) }
    }
}

pub struct GPIOE;

impl PeripheralRef for GPIOE {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOE::PTR as *mut _) }
    }
}

pub struct GPIOF;

impl PeripheralRef for GPIOF {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOF::PTR as *mut _) }
    }
}

pub struct GPIOG;

impl PeripheralRef for GPIOG {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOG::PTR as *mut _) }
    }
}

pub struct GPIOH;

impl PeripheralRef for GPIOH {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOH::PTR as *mut _) }
    }
}

pub struct GPIOI;

impl PeripheralRef for GPIOI {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOI::PTR as *mut _) }
    }
}

pub struct GPIOJ;

impl PeripheralRef for GPIOJ {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOJ::PTR as *mut _) }
    }
}

pub struct GPIOK;

impl PeripheralRef for GPIOK {
    type Output = Port;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::GPIOK::PTR as *mut _) }
    }
}

impl Peripheral for Port {
    #[inline]
    fn enable_clock(&mut self) {
        unsafe {
            let rcc = RCC::take();
            let p = self.get_port_num();
            rcc.ahb1enr().modify(#[inline] |r, w| w.bits(r.bits() | (1u32 << p)));
        }
    }

    #[inline]
    fn disable_clock(&mut self) {
        unsafe {
            let rcc = RCC::take();
            let p = self.get_port_num();
            rcc.ahb1enr().modify(#[inline] |r, w| w.bits(r.bits() & !(1u32 << p)));
        }
    }

    #[inline]
    fn reset(&mut self) {
        unsafe {
            let rcc = RCC::take();
            let p = self.get_port_num();
            rcc.ahb1rstr().modify(#[inline] |r, w| w.bits(r.bits() | (1u32 << p)));
            rcc.ahb1rstr().modify(#[inline] |r, w| w.bits(r.bits() & !(1u32 << p)));
        }
    }
}

impl Port {
    #[inline]
    pub fn pin(&self, pin: u8) -> Pin {
        assert!(pin < 16);

        unsafe {
            let addr = self as *const Self;
            Pin {
                port: addr.as_ref().unwrap(),
                pin,
            }
        }
    }

    #[inline]
    pub fn read_input_pins(&self) -> PinMask {
        self.0.idr().read().bits().into()
    }

    #[inline]
    pub fn read_output_pins(&self) -> PinMask {
        self.0.odr().read().bits().into()
    }

    #[inline]
    pub fn set_output_pins(&mut self, pins: impl Into<PinMask>) {
        let mask = u32::from(pins.into());
        unsafe {
            self.0.bsrr().write(|w| w.bits(mask));
        }
    }

    #[inline]
    pub fn reset_output_pins(&mut self, pins: impl Into<PinMask>) {
        let mask = u32::from(pins.into());
        unsafe {
            self.0.bsrr().write(|w| w.bits(mask << 16));
        }
    }

    #[inline]
    pub fn toggle_output_pins(&mut self, pins: impl Into<PinMask>) {
        let mask = u32::from(pins.into());
        unsafe {
            let r = self.0.odr().read().bits();
            self.0.bsrr().write(|w| w.bits(((r & mask) << 16) | (!r & mask)));
        }
    }

    #[inline]
    pub fn init_pins(&mut self, pins: impl Into<PinMask>, conf: PinConfig) {
        match conf {
            PinConfig::Input(pull) => self.init_input_pins(pins, pull),
            PinConfig::Output(speed) => self.init_output_pins(pins, speed),
            PinConfig::OpenDrain(speed, pull) => self.init_open_drain_pins(pins, speed, pull),
            PinConfig::Alternate(otype, speed, pull, func) =>
                self.init_alternate_pins(pins, otype, speed, pull, func),
            PinConfig::Analog => self.init_analog_pins(pins),
        }
    }

    #[inline]
    pub fn init_input_pins(&mut self, pins: impl Into<PinMask>, pull: Pull) {
        let pins = pins.into();

        self.set_pins_mode(pins, PinMode::Input);
        self.set_pins_pull(pins, pull);
    }

    #[inline]
    pub fn init_output_pins(&mut self, pins: impl Into<PinMask>, speed: Speed) {
        let pins = pins.into();

        self.set_pins_mode(pins, PinMode::Output);
        self.set_pins_output_type(pins, OutputType::PushPull);
        self.set_pins_speed(pins, speed);
        self.set_pins_pull(pins, Pull::None);
    }

    #[inline]
    pub fn init_open_drain_pins(&mut self, pins: impl Into<PinMask>, speed: Speed, pull: Pull) {
        let pins = pins.into();

        self.set_pins_mode(pins, PinMode::Output);
        self.set_pins_output_type(pins, OutputType::OpenDrain);
        self.set_pins_speed(pins, speed);
        self.set_pins_pull(pins, pull);
    }

    #[inline]
    pub fn init_alternate_pins(
        &mut self,
        pins: impl Into<PinMask>,
        otype: OutputType,
        speed: Speed,
        pull: Pull,
        func: u8
    ) {
        let pins = pins.into();

        self.set_pins_mode(pins, PinMode::Alternate);
        self.set_pins_output_type(pins, otype);
        self.set_pins_speed(pins, speed);
        self.set_pins_pull(pins, pull);
        self.set_pins_alternate_function(pins, func);
    }

    #[inline]
    pub fn init_analog_pins(&mut self, pins: impl Into<PinMask>) {
        let pins = pins.into();

        self.set_pins_mode(pins, PinMode::Analog);
        self.set_pins_pull(pins, Pull::None);
    }

    #[inline]
    pub fn set_pins_mode(&mut self, pins: impl Into<PinMask>, mode: PinMode) {
        let mask = u32::from(pins.into());

        unsafe {
            self.0.moder().modify(|r, w| {
                w.bits(Self::write_2bit_value_by_mask(r.bits(), mode as u32, mask))
            });
        }
    }

    #[inline]
    pub fn set_pins_output_type(&mut self, pins: impl Into<PinMask>, otype: OutputType) {
        let mask = u32::from(pins.into());

        unsafe {
            self.0.otyper().modify(|r, w| {
                match otype {
                    OutputType::PushPull => w.bits(r.bits() & !mask),
                    OutputType::OpenDrain => w.bits(r.bits() | mask),
                }
            });
        }
    }

    #[inline]
    pub fn set_pins_speed(&mut self, pins: impl Into<PinMask>, speed: Speed) {
        let mask = u32::from(pins.into());

        unsafe {
            self.0.ospeedr().modify(|r, w| {
                w.bits(Self::write_2bit_value_by_mask(r.bits(), speed as u32, mask))
            });
        }
    }

    #[inline]
    pub fn set_pins_pull(&mut self, pins: impl Into<PinMask>, pull: Pull) {
        let mask = u32::from(pins.into());

        unsafe {
            self.0.pupdr().modify(|r, w| {
                w.bits(Self::write_2bit_value_by_mask(r.bits(), pull as u32, mask))
            });
        }
    }

    #[inline]
    pub fn set_pins_alternate_function(&mut self, pins: impl Into<PinMask>, func: u8) {
        let mut pins = u32::from(pins.into());

        unsafe {
            self.0.afrl().modify(|r, w| {
                let mut bits = r.bits();
                let mut mask = 0b1111u32;
                let mut value = func as u32;

                for _ in 0..8 {
                    if (pins & 1) != 0 {
                        bits = (bits & !mask) | value;
                    }
                    pins >>= 1;
                    mask <<= 4;
                    value <<= 4;
                }
                w.bits(bits)
            });

            self.0.afrh().modify(|r, w| {
                let mut bits = r.bits();
                let mut mask = 0b1111u32;
                let mut value = func as u32;

                for _ in 0..8 {
                    if (pins & 1) != 0 {
                        bits = (bits & !mask) | value;
                    }
                    pins >>= 1;
                    mask <<= 4;
                    value <<= 4;
                }
                w.bits(bits)
            });
        }
    }

    #[inline]
    fn write_2bit_value_by_mask(mut bits: u32, mut value: u32, mut mask: u32) -> u32 {
        let mut mask2 = 0b11u32;

        for _ in 0..16 {
            if (mask & 1) != 0 {
                bits = (bits & !mask2) | value;
            }
            mask >>= 1;
            mask2 <<= 2;
            value <<= 2;
        }

        bits
    }

    unsafe fn get_port_num(&self) -> isize {
        addr_of!(self.0).byte_offset_from(pac::GPIOA::PTR) / 0x400
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PinMask(u16);

impl From<u16> for PinMask {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<u32> for PinMask {
    #[inline]
    fn from(value: u32) -> Self {
        Self((value & 0xffff) as _)
    }
}

impl From<PinMask> for u16 {
    #[inline]
    fn from(value: PinMask) -> Self {
        value.0
    }
}

impl From<PinMask> for u32 {
    #[inline]
    fn from(value: PinMask) -> Self {
        value.0 as _
    }
}

impl Not for PinMask {
    type Output = PinMask;

    #[inline]
    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

impl BitAnd for PinMask {
    type Output = PinMask;

    #[inline]
    fn bitand(self, Self(rhs): Self) -> Self::Output {
        Self(self.0.bitand(rhs))
    }
}

impl BitAndAssign for PinMask {
    #[inline]
    fn bitand_assign(&mut self, Self(rhs): Self) {
        self.0.bitand_assign(rhs)
    }
}

impl BitOr for PinMask {
    type Output = PinMask;

    #[inline]
    fn bitor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0.bitor(rhs))
    }
}

impl BitOrAssign for PinMask {
    #[inline]
    fn bitor_assign(&mut self, Self(rhs): Self) {
        self.0.bitor_assign(rhs)
    }
}

impl BitXor for PinMask {
    type Output = PinMask;

    #[inline]
    fn bitxor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0.bitxor(rhs))
    }
}

impl BitXorAssign for PinMask {
    #[inline]
    fn bitxor_assign(&mut self, Self(rhs): Self) {
        self.0.bitxor_assign(rhs)
    }
}

impl PinMask {
    pub const NONE: PinMask = PinMask(0);
    pub const PIN0: PinMask = PinMask(1 << 0);
    pub const PIN1: PinMask = PinMask(1 << 1);
    pub const PIN2: PinMask = PinMask(1 << 2);
    pub const PIN3: PinMask = PinMask(1 << 3);
    pub const PIN4: PinMask = PinMask(1 << 4);
    pub const PIN5: PinMask = PinMask(1 << 5);
    pub const PIN6: PinMask = PinMask(1 << 6);
    pub const PIN7: PinMask = PinMask(1 << 7);
    pub const PIN8: PinMask = PinMask(1 << 8);
    pub const PIN9: PinMask = PinMask(1 << 9);
    pub const PIN10: PinMask = PinMask(1 << 10);
    pub const PIN11: PinMask = PinMask(1 << 11);
    pub const PIN12: PinMask = PinMask(1 << 12);
    pub const PIN13: PinMask = PinMask(1 << 13);
    pub const PIN14: PinMask = PinMask(1 << 14);
    pub const PIN15: PinMask = PinMask(1 << 15);

    #[inline]
    pub fn from_pin(pin: u8) -> Self {
        assert!(pin < 16);

        (0b1u32 << pin).into()
    }

    #[inline]
    pub fn is_set(&self, pins: impl Into<Self>) -> bool {
        let mask = pins.into();
        (*self & mask) == mask
    }

    #[inline]
    pub fn set(&mut self, pins: impl Into<Self>) {
        let mask = pins.into();
        *self |= mask;
    }

    #[inline]
    pub fn reset(&mut self, pins: impl Into<Self>) {
        let mask = pins.into();
        *self &= !mask;
    }
}
