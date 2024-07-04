use core::{ cell::Cell, ops::Deref };

use crate::{ pac, PeripheralRef };

const EXTERNAL_OSC_FREQ: core::cell::Cell<u32> = Cell::new(8_000_000u32);

pub struct RCC(pac::rcc::RegisterBlock);

impl Deref for RCC {
    type Target = pac::rcc::RegisterBlock;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PeripheralRef for RCC {
    type Output = RCC;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::RCC::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

impl RCC {
    pub fn configure_system_clock(&mut self, syscfg: SystemClockConfig, buscfg: BUSConfig) {
        unsafe {
            self.apb1enr().modify(|_, w| w.pwren().set_bit());

            self.cfgr().modify(|_, w| {
                w.hpre().bits(buscfg.ahb_prescaler as _);
                w.ppre1().bits(buscfg.apb1_prescaler as _);
                w.ppre2().bits(buscfg.apb2_prescaler as _)
            });

            let flash = pac::FLASH::PTR.as_ref().unwrap();
            flash.acr().modify(|_, w| w.latency().bits(5));

            match syscfg {
                SystemClockConfig::HSI(t) => {
                    if !self.cfgr().read().sws().is_hsi() {
                        self.cr().modify(|_, w| {
                            w.hsitrim().bits(t);
                            w.hsion().set_bit()
                        });
                        while self.cr().read().hsirdy().bit_is_clear() {}

                        self.cfgr().modify(|_, w| w.sw().hsi());
                        while !self.cfgr().read().sws().is_hsi() {}
                    }
                }
                SystemClockConfig::HSE(_) => {
                    if !self.cfgr().read().sws().is_hse() {
                        self.cr().modify(|_, w| w.hseon().set_bit());
                        while self.cr().read().hserdy().bit_is_clear() {}

                        self.cfgr().modify(|_, w| w.sw().hse());
                        while !self.cfgr().read().sws().is_hse() {}
                    }
                }
                SystemClockConfig::PLL(config) => {
                    if !self.cfgr().read().sws().is_pll() {
                        match config.clock_source {
                            PLLClockSource::HSI => {
                                self.cr().modify(|_, w| w.hsion().set_bit());
                                while self.cr().read().hsirdy().bit_is_clear() {}
                            }
                            PLLClockSource::HSE => {
                                self.cr().modify(|_, w| w.hseon().set_bit());
                                while self.cr().read().hserdy().bit_is_clear() {}
                            }
                        }
                        self.pllcfgr().write(|w| {
                            w.pllsrc().bit(config.clock_source == PLLClockSource::HSE);
                            w.pllm().bits(config.pllm);
                            w.plln().bits(config.plln);
                            w.pllp().bits(config.system_clock_div_factor as _);
                            w.pllq().bits(config.pllq)
                        });
                        self.cr().modify(|_, w| w.pllon().set_bit());
                        while self.cr().read().pllrdy().bit_is_clear() {}

                        self.cfgr().modify(|_, w| w.sw().pll());
                        while !self.cfgr().read().sws().is_pll() {}
                    }
                }
            }
        }
    }

    pub fn configure_pllsai(&mut self, config: PLLSAIConfig) {
        unsafe {
            self.cr().modify(|_, w| w.pllsaion().clear_bit());
            while self.cr().read().pllsairdy().bit_is_set() {}

            self.pllsaicfgr().write(|w| {
                w.pllsain().bits(config.pllsain as _);
                w.pllsaiq().bits(config.pllsaiq as _);
                w.pllsair().bits(config.pllsair as _)
            });
            self.dckcfgr().modify(|_, w| w.pllsaidivr().bits(config.lcd_div_factor as _));

            self.cr().modify(|_, w| w.pllsaion().set_bit());
            while self.cr().read().pllsairdy().bit_is_clear() {}
        }
    }

    #[inline]
    pub fn sysclock_clock_source(&self) -> SystemClockSource {
        SystemClockSource::from_bits(self.cfgr().read().sws().bits() as _)
    }

    #[inline]
    pub fn pll_clock_source(&self) -> PLLClockSource {
        PLLClockSource::from_bits(self.pllcfgr().read().pllsrc().bit() as _)
    }

    #[inline]
    pub fn pll_division_factor(&self) -> u8 {
        self.pllcfgr().read().pllm().bits()
    }

    #[inline]
    pub fn pll_multiplication_factor(&self) -> u16 {
        self.pllcfgr().read().plln().bits()
    }

    #[inline]
    pub fn pll_sysclock_division_factor(&self) -> PLLSysClockDivisionFactor {
        PLLSysClockDivisionFactor::from_bits(self.pllcfgr().read().pllp().bits() as _)
    }

    #[inline]
    pub fn ahb_prescaler(&self) -> AHBPrescaler {
        AHBPrescaler::from_bits(self.cfgr().read().hpre().bits() as _)
    }

    #[inline]
    pub fn apb1_prescaler(&self) -> APBPrescaler {
        APBPrescaler::from_bits(self.cfgr().read().ppre1().bits() as _)
    }

    #[inline]
    pub fn apb2_prescaler(&self) -> APBPrescaler {
        APBPrescaler::from_bits(self.cfgr().read().ppre2().bits() as _)
    }

    #[inline]
    pub fn sysclk_freq(&self) -> u32 {
        match self.sysclock_clock_source() {
            SystemClockSource::HSI => 16_000_000u32,
            SystemClockSource::HSE => EXTERNAL_OSC_FREQ.get(),
            SystemClockSource::PLL => {
                let freq = match self.pll_clock_source() {
                    PLLClockSource::HSI => 16_000_000u32,
                    PLLClockSource::HSE => EXTERNAL_OSC_FREQ.get(),
                };

                let pllp = match self.pll_sysclock_division_factor() {
                    PLLSysClockDivisionFactor::DividedBy2 => 2,
                    PLLSysClockDivisionFactor::DividedBy4 => 4,
                    PLLSysClockDivisionFactor::DividedBy6 => 6,
                    PLLSysClockDivisionFactor::DividedBy8 => 8,
                };

                freq.saturating_div(self.pll_division_factor() as u32)
                    .saturating_mul(self.pll_multiplication_factor() as u32)
                    .saturating_div(pllp)
            }
        }
    }

    #[inline]
    pub fn hclk_freq(&self) -> u32 {
        let ahb_div = match self.ahb_prescaler() {
            AHBPrescaler::NotDivided => 1,
            AHBPrescaler::DividedBy2 => 2,
            AHBPrescaler::DividedBy4 => 4,
            AHBPrescaler::DividedBy8 => 8,
            AHBPrescaler::DividedBy16 => 16,
            AHBPrescaler::DividedBy64 => 64,
            AHBPrescaler::DividedBy128 => 128,
            AHBPrescaler::DividedBy256 => 256,
            AHBPrescaler::DividedBy512 => 512,
        };

        self.sysclk_freq() / ahb_div
    }

    #[inline]
    pub fn pclk1_freq(&self) -> u32 {
        let apb1_div = match self.apb1_prescaler() {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::DividedBy2 => 2,
            APBPrescaler::DividedBy4 => 4,
            APBPrescaler::DividedBy8 => 8,
            APBPrescaler::DividedBy16 => 16,
        };

        self.hclk_freq() / apb1_div
    }

    #[inline]
    pub fn pclk2_freq(&self) -> u32 {
        let apb2_div = match self.apb2_prescaler() {
            APBPrescaler::NotDivided => 1,
            APBPrescaler::DividedBy2 => 2,
            APBPrescaler::DividedBy4 => 4,
            APBPrescaler::DividedBy8 => 8,
            APBPrescaler::DividedBy16 => 16,
        };

        self.hclk_freq() / apb2_div
    }
}

pub enum SystemClockConfig {
    HSI(u8),
    HSE(u16),
    PLL(PLLConfig),
}

pub struct PLLConfig {
    pub clock_source: PLLClockSource,
    pub pllm: u8,
    pub plln: u16,
    pub pllq: u8,
    pub system_clock_div_factor: PLLSysClockDivisionFactor,
}

pub struct PLLSAIConfig {
    pub pllsain: u16,
    pub pllsaiq: u8,
    pub pllsair: u8,
    pub lcd_div_factor: LCDClockDivisionFactor,
}

pub struct BUSConfig {
    pub ahb_prescaler: AHBPrescaler,
    pub apb1_prescaler: APBPrescaler,
    pub apb2_prescaler: APBPrescaler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemClockSource {
    HSI = 0b00,
    HSE = 0b01,
    PLL = 0b10,
}

impl SystemClockSource {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b00 => Self::HSI,
            0b01 => Self::HSE,
            0b10 => Self::PLL,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLLClockSource {
    HSI = 0b0,
    HSE = 0b1,
}

impl PLLClockSource {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b0 => Self::HSI,
            0b1 => Self::HSE,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PLLSysClockDivisionFactor {
    DividedBy2 = 0b00,
    DividedBy4 = 0b01,
    DividedBy6 = 0b10,
    DividedBy8 = 0b11,
}

impl PLLSysClockDivisionFactor {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b00 => Self::DividedBy2,
            0b01 => Self::DividedBy4,
            0b10 => Self::DividedBy6,
            0b11 => Self::DividedBy8,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LCDClockDivisionFactor {
    DividedBy2 = 0b00,
    DividedBy4 = 0b01,
    DividedBy8 = 0b10,
    DividedBy16 = 0b11,
}

impl LCDClockDivisionFactor {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b00 => Self::DividedBy2,
            0b01 => Self::DividedBy4,
            0b10 => Self::DividedBy8,
            0b11 => Self::DividedBy16,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AHBPrescaler {
    NotDivided = 0b0000,
    DividedBy2 = 0b1000,
    DividedBy4 = 0b1001,
    DividedBy8 = 0b1010,
    DividedBy16 = 0b1011,
    DividedBy64 = 0b1100,
    DividedBy128 = 0b1101,
    DividedBy256 = 0b1110,
    DividedBy512 = 0b1111,
}

impl AHBPrescaler {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b1000 => Self::DividedBy2,
            0b1001 => Self::DividedBy4,
            0b1010 => Self::DividedBy8,
            0b1011 => Self::DividedBy16,
            0b1100 => Self::DividedBy64,
            0b1101 => Self::DividedBy128,
            0b1110 => Self::DividedBy256,
            0b1111 => Self::DividedBy512,
            _ => Self::NotDivided,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum APBPrescaler {
    NotDivided = 0b000,
    DividedBy2 = 0b100,
    DividedBy4 = 0b101,
    DividedBy8 = 0b110,
    DividedBy16 = 0b111,
}

impl APBPrescaler {
    pub fn from_bits(val: u32) -> Self {
        match val {
            0b100 => Self::DividedBy2,
            0b101 => Self::DividedBy4,
            0b110 => Self::DividedBy8,
            0b111 => Self::DividedBy16,
            _ => Self::NotDivided,
        }
    }

    pub fn into_bits(val: Self) -> u32 {
        match val {
            Self::NotDivided => 0,
            Self::DividedBy2 => 0b100,
            Self::DividedBy4 => 0b101,
            Self::DividedBy8 => 0b110,
            Self::DividedBy16 => 0b111,
        }
    }
}
