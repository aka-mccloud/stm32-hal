use crate::PeripheralRef;

pub struct RCC(pac::rcc::RegisterBlock);

impl PeripheralRef for RCC {
    type Output = RCC;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::RCC::ptr() as *mut _) }
    }
}

impl RCC {
    pub fn configure_system_clock(&mut self, syscfg: SystemClockConfig, buscfg: BUSConfig) {
        unsafe {
            self.0.apb1enr.modify(|_, w| w.pwren().set_bit());
            
            self.0.cfgr.modify(|_, w| {
                w.hpre().bits(buscfg.ahb_prescaler as _);
                w.ppre1().bits(buscfg.apb1_prescaler as _);
                w.ppre2().bits(buscfg.apb2_prescaler as _)
            });

            let flash = &mut *(pac::FLASH::ptr() as *mut pac::flash::RegisterBlock);
            flash.acr.modify(|_, w| w.latency().bits(5));
            
            match syscfg {
                SystemClockConfig::HSI(t) => {
                    if !self.0.cfgr.read().sws().is_hsi() {
                        self.0.cr.modify(|_, w| {
                            w.hsitrim().bits(t);
                            w.hsion().set_bit()
                        });
                        while self.0.cr.read().hsirdy().bit_is_clear() {}

                        self.0.cfgr.modify(|_, w| w.sw().hsi());
                        while !self.0.cfgr.read().sws().is_hsi() {}
                    }
                }
                SystemClockConfig::HSE(_) => {
                    if !self.0.cfgr.read().sws().is_hse() {
                        self.0.cr.modify(|_, w| w.hseon().set_bit());
                        while self.0.cr.read().hserdy().bit_is_clear() {}

                        self.0.cfgr.modify(|_, w| w.sw().hse());
                        while !self.0.cfgr.read().sws().is_hse() {}
                    }
                }
                SystemClockConfig::PLL(config) => {
                    if !self.0.cfgr.read().sws().is_pll() {
                        match config.clock_source {
                            PLLClockSource::HSI => {
                                self.0.cr.modify(|_, w| w.hsion().set_bit());
                                while self.0.cr.read().hsirdy().bit_is_clear() {}
                            }
                            PLLClockSource::HSE => {
                                self.0.cr.modify(|_, w| w.hseon().set_bit());
                                while self.0.cr.read().hserdy().bit_is_clear() {}
                            }
                        }
                        self.0.pllcfgr.write(|w| {
                            w.pllsrc().bit(config.clock_source == PLLClockSource::HSE);
                            w.pllm().bits(config.pllm);
                            w.plln().bits(config.plln);
                            w.pllp().bits(config.system_clock_div_factor as _);
                            w.pllq().bits(config.pllq)
                        });
                        self.0.cr.modify(|_, w| w.pllon().set_bit());
                        while self.0.cr.read().pllrdy().bit_is_clear() {}

                        self.0.cfgr.modify(|_, w| w.sw().pll());
                        while !self.0.cfgr.read().sws().is_pll() {}
                    }
                }
            }
        }
    }

    pub fn configure_pllsai(&mut self, config: PLLSAIConfig) {
        unsafe {
            self.0.cr.modify(|_, w| w.pllsaion().clear_bit());
            while self.0.cr.read().pllsairdy().bit_is_set() {}

            self.0.pllsaicfgr.write(|w| {
                w.pllsain().bits(config.pllsain as _);
                w.pllsaiq().bits(config.pllsaiq as _);
                w.pllsair().bits(config.pllsair as _)
            });
            self.0.dckcfgr.modify(|_, w| w.pllsaidivr().bits(config.lcd_div_factor as _));

            self.0.cr.modify(|_, w| w.pllsaion().set_bit());
            while self.0.cr.read().pllsairdy().bit_is_clear() {}
        }
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
