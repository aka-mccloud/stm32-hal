use core::ops::Deref;

use crate::{ pac::{ self, interrupt }, rcc::RCC, InterruptHandler, Peripheral, PeripheralRef };

pub struct LTDC(pac::ltdc::RegisterBlock);

impl Deref for LTDC {
    type Target = pac::ltdc::RegisterBlock;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PeripheralRef for LTDC {
    type Output = LTDC;

    fn take() -> &'static mut Self::Output {
        unsafe { (pac::LTDC::PTR as *mut Self::Output).as_mut().unwrap() }
    }
}

impl Peripheral for LTDC {
    fn enable_clock(&mut self) {
        RCC::take()
            .apb2enr()
            .modify(|_, w| w.ltdcen().set_bit());
    }

    fn disable_clock(&mut self) {
        RCC::take()
            .apb2enr()
            .modify(|_, w| w.ltdcen().clear_bit());
    }

    fn reset(&mut self) {
        let rcc = RCC::take();
        rcc.apb2rstr().modify(|_, w| w.ltdcrst().set_bit());
        rcc.apb2rstr().modify(|_, w| w.ltdcrst().clear_bit());
    }
}

impl LTDC {
    pub fn enable(&mut self) {
        self.0.gcr().modify(|_, w| w.ltdcen().set_bit());
    }

    pub fn disable(&mut self) {
        self.0.gcr().modify(|_, w| w.ltdcen().clear_bit());
    }

    pub fn is_enabled(&self) -> bool {
        self.0.gcr().read().ltdcen().bit()
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
        self.ier().write(|w| {
            w.lie().set_bit();
            w.fuie().set_bit();
            w.terrie().set_bit();
            w.rrie().set_bit()
        });

        unsafe {
            pac::NVIC::unmask(interrupt::LCD_TFT);
            pac::NVIC::unmask(interrupt::LCD_TFT_1);
        }

        // reload the shadow registers

        // enable LCD-TFT controller
        self.enable();
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
            self.0.bccr().modify(|_, w| { w.bits(color.into_rgb888()) });
        }
    }

    pub fn enable_dither(&mut self, color: Color) {
        unsafe {
            self.0.gcr().modify(|_, w| {
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
        self.0.gcr().modify(|_, w| { w.den().clear_bit() })
    }

    pub fn layer1_enable(&mut self) {
        self.0
            .layer1()
            .cr()
            .modify(|_, w| w.len().set_bit())
    }

    pub fn layer1_disable(&mut self) {
        self.0
            .layer1()
            .cr()
            .modify(|_, w| w.len().clear_bit())
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
        self.layer_configure(
            self.0.layer1(),
            x,
            y,
            width,
            height,
            pixel_format,
            default_color,
            image_buffer_address
        );

        // enable layer
        self.layer1_enable();

        // reload shadow registers
        self.0.srcr().modify(|_, w| w.imr().set_bit());
    }

    pub fn layer2_enable(&mut self) {
        self.0
            .layer2()
            .cr()
            .modify(|_, w| w.len().set_bit())
    }

    pub fn layer2_disable(&mut self) {
        self.0
            .layer2()
            .cr()
            .modify(|_, w| w.len().clear_bit())
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
        self.layer_configure(
            self.0.layer2(),
            x,
            y,
            width,
            height,
            pixel_format,
            default_color,
            image_buffer_address
        );

        // enable layer
        self.layer2_enable();

        // reload shadow registers
        self.0.srcr().modify(|_, w| w.imr().set_bit());
    }

    fn layer_configure(
        &self,
        layer: &pac::ltdc::LAYER,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        pixel_format: PixelFormat,
        default_color: Color,
        image_buffer_address: *const u8
    ) {
        // configure layer window
        let ahbp = self.0.bpcr().read().ahbp().bits();
        layer.whpcr().write(|w| {
            w.whstpos().set(ahbp + x + 1);
            w.whsppos().set(ahbp + x + width)
        });

        let avbp = self.0.bpcr().read().avbp().bits();
        layer.wvpcr().write(|w| {
            w.wvstpos().set(avbp + y + 1);
            w.wvsppos().set(avbp + y + height)
        });

        // set pixel format
        layer.pfcr().write(|w| w.pf().set(pixel_format as _));

        // set color frame buffer start address
        layer.cfbar().write(|w| w.set(image_buffer_address as u32));

        // set line length and pitch
        let pitch = width * (pixel_format.byte_len() as u16);
        layer.cfblr().write(|w| {
            w.cfbll().set(pitch + 3);
            w.cfbp().set(pitch)
        });

        // set number of lines
        layer.cfblnr().write(|w| w.cfblnbr().set(height));

        // load CLUT if needed

        // set alpha constant
        layer.cacr().modify(|_, w| w.consta().set(255));

        // configure default color and blending factors if needed
        layer.dccr().write(|w| {
            w.dcalpha().set(default_color.0);
            w.dcred().set(default_color.1);
            w.dcgreen().set(default_color.2);
            w.dcblue().set(default_color.3)
        });
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
        self.0.sscr().modify(|_, w| {
            w.hsw().set(hsw - 1);
            w.vsh().set(vsh - 1)
        });
        self.0.bpcr().write(|w| {
            w.ahbp().set(hsw + (hbp as u16) - 1);
            w.avbp().set(vsh + (vbp as u16) - 1)
        });
        self.0.awcr().write(|w| {
            w.aaw().set(hsw + (hbp as u16) + width - 1);
            w.aah().set(vsh + (vbp as u16) + height - 1)
        });
        self.0.twcr().write(|w| {
            w.totalw().set(hsw + (hbp as u16) + width + (hfp as u16) - 1);
            w.totalh().set(vsh + (vbp as u16) + height + (vfp as u16) - 1)
        });
    }

    fn set_signal_polarity(
        &mut self,
        pcpol: PixelClockPolarity,
        depol: Polarity,
        vspol: Polarity,
        hspol: Polarity
    ) {
        self.0.gcr().modify(|_, w| {
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

/// LTDC global event interrupt
#[interrupt]
fn LCD_TFT() {
    let ltdc = LTDC::take();
    if ltdc.isr().read().rrif().bit_is_set() {
        ltdc.ier().modify(|_, w| w.rrie().clear_bit());
        ltdc.icr().write(|w| w.crrif().bit(true));

        unsafe {
            (IRQ_HANDLERS[REGISTER_RELOAD_HANDLER])();
        }
    }

    if ltdc.isr().read().lif().bit_is_set() {
        ltdc.ier().modify(|_, w| w.lie().clear_bit());
        ltdc.icr().write(|w| w.clif().bit(true));

        unsafe {
            (IRQ_HANDLERS[LINE_INTERRUPT_HANDLER])();
        }
    }
}

/// LTDC global error interrupt
#[interrupt]
fn LCD_TFT_1() {
    let ltdc = LTDC::take();
    if ltdc.isr().read().terrif().bit_is_set() {
        ltdc.ier().modify(|_, w| w.terrie().clear_bit());
        ltdc.icr().write(|w| w.cterrif().bit(true));

        unsafe {
            (IRQ_HANDLERS[TRANSFER_ERROR_HANDLER])();
        }
    }

    if ltdc.isr().read().fuif().bit_is_set() {
        ltdc.ier().modify(|_, w| w.fuie().clear_bit());
        ltdc.icr().write(|w| w.cfuif().bit(true));

        unsafe {
            (IRQ_HANDLERS[FIFO_UNDERRUN_HANDLER])();
        }
    }
}
