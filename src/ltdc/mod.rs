use cortex_m::{ interrupt::Nr, peripheral::nvic };
use pac::interrupt;
use stm32f429::NVIC;

use crate::{ InterruptHandler, Peripheral, PeripheralRef };

pub struct LTDC(pac::ltdc::RegisterBlock);

impl PeripheralRef for LTDC {
    type Output = LTDC;

    fn take() -> &'static mut Self::Output {
        unsafe { &mut *(pac::LTDC::ptr() as *mut _) }
    }
}

impl Peripheral for LTDC {
    fn enable_clock(&mut self) {
        unsafe {
            let rcc = &*pac::RCC::ptr();
            rcc.apb2enr.modify(|_, w| w.ltdcen().set_bit());
        }
    }

    fn disable_clock(&mut self) {
        unsafe {
            let rcc = &*pac::RCC::ptr();
            rcc.apb2enr.modify(|_, w| w.ltdcen().clear_bit());
        }
    }

    fn reset(&mut self) {
        unsafe {
            let rcc = &*pac::RCC::ptr();
            rcc.apb2rstr.modify(|_, w| w.ltdcrst().set_bit());
            rcc.apb2rstr.modify(|_, w| w.ltdcrst().clear_bit());
        }
    }
}

impl LTDC {
    pub fn enable(&mut self) {
        self.0.gcr.modify(|_, w| w.ltdcen().set_bit());
    }

    pub fn disable(&mut self) {
        self.0.gcr.modify(|_, w| w.ltdcen().clear_bit());
    }

    pub fn is_enabled(&self) -> bool {
        self.0.gcr.read().ltdcen().bit()
    }

    pub fn init(&mut self, conf: LTDCConfig) {
        // enable clock
        self.enable_clock();
        self.reset();

        // configure required pixel clock (from arg)

        // configure synchronous signals and clock polarity
        self.set_signal_polarity(
            conf.pixel_clock_polarity,
            conf.data_enable_polarity,
            conf.vertical_sync_polarity,
            conf.horizontal_sync_polarity
        );

        let mut val = self.0.gcr.read().bits();
        val |= 1;

        // configure synchronous timings
        self.set_sync_timings(
            conf.active_width,
            conf.active_height,
            conf.horizontal_sync,
            conf.vertical_sync,
            conf.horizontal_back_porch,
            conf.vertical_back_porch,
            conf.horizontal_front_porch,
            conf.vertical_front_porch
        );

        // configure background color
        self.set_background_color(conf.background_color);

        // enable interrupts
        // self.regs.ier.write(|w| {
        //     w.lie().set_bit();
        //     w.fuie().set_bit();
        //     w.terrie().set_bit();
        //     w.rrie().set_bit()
        // });

        unsafe {
            let lcd_tft_irqn = interrupt::LCD_TFT.nr();
            let lcd_tft1_irqn = interrupt::LCD_TFT_1.nr();
            let regs = &mut *(NVIC::ptr() as *mut nvic::RegisterBlock);
            regs.iser[(lcd_tft_irqn as usize) / 32].modify(|r| r | (0b1u32 << lcd_tft_irqn % 32));
            regs.iser[(lcd_tft1_irqn as usize) / 32].modify(|r| r | (0b1u32 << lcd_tft1_irqn % 32));
        }

        // reload the shadow registers

        // enable LCD-TFT controller
        self.enable();
        unsafe {
        self.0.gcr.write(|w| w.bits(val));
        }
    }

    pub fn set_register_reload_event_handler(&mut self, f: InterruptHandler) {
        unsafe {
            IRQ_HANDLERS[REGISTER_RELOAD_HANDLER] = f;
        }
    }

    pub fn set_line_event_handler(&mut self, f: InterruptHandler) {
        unsafe {
            IRQ_HANDLERS[LINE_INTERRUPT_HANDLER] = f;
        }
    }

    pub fn set_transfer_error_handler(&mut self, f: InterruptHandler) {
        unsafe {
            IRQ_HANDLERS[TRANSFER_ERROR_HANDLER] = f;
        }
    }

    pub fn set_fifo_underrun_handler(&mut self, f: InterruptHandler) {
        unsafe {
            IRQ_HANDLERS[FIFO_UNDERRUN_HANDLER] = f;
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        unsafe {
            self.0.bccr.modify(|_, w| { w.bc().bits(color.into_rgb888()) });
        }
    }

    pub fn enable_dither(&mut self, color: Color) {
        unsafe {
            self.0.gcr.modify(|_, w| {
                w.bits(
                    (((color.1 as u32) & 0b111) << 12) |
                        (((color.2 as u32) & 0b111) << 8) |
                        (((color.3 as u32) & 0b111) << 4)
                );
                w.den().set_bit()
            })
        }
    }

    pub fn disable_dither(&mut self) {
        self.0.gcr.modify(|_, w| { w.den().clear_bit() })
    }

    pub fn layer1_enable(&mut self) {
        self.0.l1cr.modify(|_, w| w.len().set_bit())
    }

    pub fn layer1_disable(&mut self) {
        self.0.l1cr.modify(|_, w| w.len().clear_bit())
    }

    pub fn layer1_configure(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        pixel_format: PixelFormat,
        default_color: Color,
        image_buffer_address: *const u8
    ) {
        unsafe {
            // configure layer window
            let ahbp = self.0.bpcr.read().ahbp().bits();
            self.0.l1whpcr.write(|w| {
                w.whstpos().bits(ahbp + x + 1);
                w.whsppos().bits(ahbp + x + width)
            });

            let avbp = self.0.bpcr.read().avbp().bits();
            self.0.l1wvpcr.write(|w| {
                w.wvstpos().bits(avbp + y + 1);
                w.wvsppos().bits(avbp + y + height)
            });

            // set pixel format
            self.0.l1pfcr.write(|w| w.pf().bits(pixel_format as _));

            // set color frame buffer start address
            self.0.l1cfbar.write(|w| w.bits(image_buffer_address as u32));

            // set line length and pitch
            let pitch = width * (pixel_format.byte_len() as u16);
            self.0.l1cfblr.write(|w| {
                w.cfbll().bits(pitch + 3);
                w.cfbp().bits(pitch)
            });

            // set number of lines
            self.0.l1cfblnr.write(|w| w.cfblnbr().bits(height));

            // load CLUT if needed

            // set alpha constant
            self.0.l1cacr.modify(|_, w| w.consta().bits(255));

            // configure default color and blending factors if needed
            self.0.l1dccr.write(|w| w.bits(default_color.into_argb8888()));

            // enable layer
            self.0.l1cr.modify(|_, w| w.len().set_bit());

            // reload shadow registers
            self.0.srcr.modify(|_, w| w.imr().set_bit());
        }
    }

    pub fn layer2_enable(&mut self) {
        self.0.l2cr.modify(|_, w| w.len().set_bit())
    }

    pub fn layer2_disable(&mut self) {
        self.0.l2cr.modify(|_, w| w.len().clear_bit())
    }

    pub fn layer2_configure(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        pixel_format: PixelFormat,
        default_color: Color,
        image_buffer_address: *const u8
    ) {
        unsafe {
            // configure layer window
            let ahbp = self.0.bpcr.read().ahbp().bits();
            self.0.l2whpcr.write(|w| {
                w.whstpos().bits(ahbp + x + 1);
                w.whsppos().bits(ahbp + x + width)
            });

            let avbp = self.0.bpcr.read().avbp().bits();
            self.0.l2wvpcr.write(|w| {
                w.wvstpos().bits(avbp + y + 1);
                w.wvsppos().bits(avbp + y + height)
            });

            // set pixel format
            self.0.l2pfcr.write(|w| w.pf().bits(pixel_format as _));

            // set color frame buffer start address
            self.0.l2cfbar.write(|w| w.bits(image_buffer_address as u32));

            // set line length and pitch
            let pitch = width * (pixel_format.byte_len() as u16);
            self.0.l2cfblr.write(|w| {
                w.cfbll().bits(pitch + 3);
                w.cfbp().bits(pitch)
            });

            // set number of lines
            self.0.l2cfblnr.write(|w| w.cfblnbr().bits(height));

            // load CLUT if needed

            // set alpha constant
            self.0.l2cacr.modify(|_, w| w.consta().bits(255));

            // configure default color and blending factors if needed
            self.0.l2dccr.write(|w| w.bits(default_color.into_argb8888()));

            // enable layer
            self.0.l2cr.modify(|_, w| w.len().set_bit());

            // reload shadow registers
            self.0.srcr.modify(|_, w| w.imr().set_bit());
        }
    }

    fn set_sync_timings(
        &mut self,
        width: u16,
        height: u16,
        hsw: u16,
        vsh: u16,
        hbp: u16,
        vbp: u16,
        hfp: u16,
        vfp: u16
    ) {
        unsafe {
            self.0.sscr.modify(|_, w| {
                w.hsw().bits(hsw - 1);
                w.vsh().bits(vsh - 1)
            });
            self.0.bpcr.write(|w| {
                w.ahbp().bits(hsw + (hbp as u16) - 1);
                w.avbp().bits(vsh + (vbp as u16) - 1)
            });
            self.0.awcr.write(|w| {
                w.aav().bits(hsw + (hbp as u16) + width - 1);
                w.aah().bits(vsh + (vbp as u16) + height - 1)
            });
            self.0.twcr.write(|w| {
                w.totalw().bits(hsw + (hbp as u16) + width + (hfp as u16) - 1);
                w.totalh().bits(vsh + (vbp as u16) + height + (vfp as u16) - 1)
            });
        }
    }

    fn set_signal_polarity(
        &mut self,
        pcpol: PixelClockPolarity,
        depol: Polarity,
        vspol: Polarity,
        hspol: Polarity
    ) {
        self.0.gcr.modify(|_, w| {
            w.pcpol().bit(pcpol == PixelClockPolarity::Inverted);
            w.depol().bit(depol == Polarity::ActiveHigh);
            w.vspol().bit(vspol == Polarity::ActiveHigh);
            w.hspol().bit(hspol == Polarity::ActiveHigh)
        });
    }
}

pub struct LTDCConfig {
    /// Horizontal Synchonization Polarity
    pub horizontal_sync_polarity: Polarity,

    /// Vertical Synchonization Polarity
    pub vertical_sync_polarity: Polarity,

    /// Data Enable Polarity
    pub data_enable_polarity: Polarity,

    /// Pixel Clock Polarity
    pub pixel_clock_polarity: PixelClockPolarity,

    /// Horizontal Synchronization Width
    pub horizontal_sync: u16,

    /// Vertical Synchronization Height
    pub vertical_sync: u16,

    /// Horizontal Back Porch
    pub horizontal_back_porch: u16,

    /// Vertical Back Porch
    pub vertical_back_porch: u16,

    /// Active Pixels Width
    pub active_width: u16,

    /// Active Pixels Height
    pub active_height: u16,

    /// Horizontal Front Porch
    pub horizontal_front_porch: u16,

    /// Vertical Front Porch
    pub vertical_front_porch: u16,

    /// Background Color
    pub background_color: Color,
}

/// Color(A,R,G,B)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl Color {
    pub fn into_argb8888(self) -> u32 {
        let Color(a, r, g, b) = self;
        ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }

    pub fn into_rgb888(self) -> u32 {
        let Color(_, r, g, b) = self;
        ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    ARGB8888 = 0b000,
    RGB888 = 0b001,
    RGB565 = 0b010,
    ARGB1555 = 0b011,
    ARGB4444 = 0b100,
    L8 = 0b101,
    AL44 = 0b110,
    AL88 = 0b111,
}

impl PixelFormat {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b000 => Self::ARGB8888,
            0b001 => Self::RGB888,
            0b010 => Self::RGB565,
            0b011 => Self::ARGB1555,
            0b100 => Self::ARGB4444,
            0b101 => Self::L8,
            0b110 => Self::AL44,
            0b111 => Self::AL88,
            _ => panic!(),
        }
    }

    pub fn byte_len(&self) -> u8 {
        match self {
            PixelFormat::ARGB8888 => 4,
            PixelFormat::RGB888 => 3,
            PixelFormat::RGB565 => 2,
            PixelFormat::ARGB1555 => 2,
            PixelFormat::ARGB4444 => 2,
            PixelFormat::L8 => 1,
            PixelFormat::AL44 => 1,
            PixelFormat::AL88 => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelClockPolarity {
    Normal = 0b0,
    Inverted = 0b1,
}

impl PixelClockPolarity {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b0 => Self::Normal,
            0b1 => Self::Inverted,
            _ => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    ActiveLow = 0b0,
    ActiveHigh = 0b1,
}

impl Polarity {
    pub fn from_bits(value: u32) -> Self {
        match value {
            0b0 => Self::ActiveLow,
            0b1 => Self::ActiveHigh,
            _ => panic!(),
        }
    }
}

fn default_handler() {}

static mut IRQ_HANDLERS: [InterruptHandler; 4] = [default_handler; 4];

const LINE_INTERRUPT_HANDLER: usize = 0;
const FIFO_UNDERRUN_HANDLER: usize = 1;
const TRANSFER_ERROR_HANDLER: usize = 2;
const REGISTER_RELOAD_HANDLER: usize = 3;

/// LTDC global interrupt
#[interrupt]
fn LCD_TFT() {
    let ltdc = LTDC::take();
    if ltdc.0.isr.read().rrif().bit_is_set() {
        ltdc.0.ier.modify(|_, w| w.rrie().clear_bit());
        ltdc.0.icr.write(|w| w.crrif().set_bit());

        unsafe {
            (IRQ_HANDLERS[REGISTER_RELOAD_HANDLER])();
        }
    }

    if ltdc.0.isr.read().lif().bit_is_set() {
        ltdc.0.ier.modify(|_, w| w.lie().clear_bit());
        ltdc.0.icr.write(|w| w.clif().set_bit());

        unsafe {
            (IRQ_HANDLERS[LINE_INTERRUPT_HANDLER])();
        }
    }
    
}

/// LTDC global error interrupt
#[interrupt]
fn LCD_TFT_1() {
    let ltdc = LTDC::take();
    if ltdc.0.isr.read().terrif().bit_is_set() {
        ltdc.0.ier.modify(|_, w| w.terrie().clear_bit());
        ltdc.0.icr.write(|w| w.cterrif().set_bit());

        unsafe {
            (IRQ_HANDLERS[TRANSFER_ERROR_HANDLER])();
        }
    }

    if ltdc.0.isr.read().fuif().bit_is_set() {
        ltdc.0.ier.modify(|_, w| w.fuie().clear_bit());
        ltdc.0.icr.write(|w| w.cfuif().set_bit());

        unsafe {
            (IRQ_HANDLERS[FIFO_UNDERRUN_HANDLER])();
        }
    }
}
