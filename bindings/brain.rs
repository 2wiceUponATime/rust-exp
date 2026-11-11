use core::{ffi::c_char, ptr::NonNull};

use alloc::ffi::NulError;

use crate::{
    CStrArg, IntoCStrPtr, as_ptr, into_callback, opaque,
    thread::{Mutex, MutexGuard},
    units::{Current, Units, Voltage},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RenderError;

static BRAIN: Mutex<Option<&'static Brain>> = Mutex::new(None);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FontType {
    #[default]
    Mono20 = 0,
    Mono30,
    Mono40,
    Mono60,
    Prop20,
    Prop30,
    Prop40,
    Prop60,
    Mono15,
    Mono12,
    Cjk16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    transparent: bool,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, transparent: bool) -> Self {
        Self {
            r,
            g,
            b,
            transparent,
        }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, false)
    }

    pub const fn from_hsv(h: u8, s: u8, v: u8) -> Self {
        let h6 = h as u16 * 6;
        let sector = h6 >> 5;
        let f = h6 & 0xFF;
        let (s, v16) = (s as u16, v as u16);
        let p = (v16 * (255 - s) / 255) as u8;
        let q = (v16 * (255 - s * f / 255) / 255) as u8;
        let t = (v16 * (255 - s * (255 - f) / 255) / 255) as u8;

        match sector {
            0 => Self::rgb(v, t, p),
            1 => Self::rgb(q, v, p),
            2 => Self::rgb(p, v, t),
            3 => Self::rgb(p, q, v),
            4 => Self::rgb(t, p, v),
            _ => Self::rgb(v, p, q),
        }
    }

    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, true);
}

#[derive(Clone, Copy)]
pub struct Style {
    pub font: FontType,
    pub pen_width: u32,
    pub origin: (i32, i32),
    pub pen_color: Color,
    pub fill_color: Color,
}

impl Style {
    pub const DEFAULT: Self = Self {
        font: FontType::Mono20,
        pen_width: 1,
        origin: (0, 0),
        pen_color: Color::WHITE,
        fill_color: Color::WHITE,
    };
}

impl Default for Style {
    fn default() -> Self {
        Self::DEFAULT
    }
}

opaque!(pub Brain);

impl Brain {
    as_ptr!();

    #[cpp(func = "new brain")]
    fn new_ptr() -> *mut Brain;

    fn new() -> NonNull<Self> {
        NonNull::new(Self::new_ptr()).expect("failed to allocate Brain")
    }

    pub fn get() -> &'static Self {
        let mut guard = BRAIN.lock();
        guard.get_or_insert_with(|| unsafe { Self::new().as_ref() })
    }

    #[cpp(body = "return &self->buttonUp;")]
    fn button_up_ptr(&self) -> *mut BrainButton;

    pub fn button_up(&self) -> &BrainButton {
        unsafe { self.button_up_ptr().as_ref_unchecked() }
    }

    #[cpp(body = "return &self->Screen;")]
    fn lcd_ptr(&self) -> *mut BrainLcd;

    pub fn lcd(&self) -> &BrainLcd {
        unsafe { self.lcd_ptr().as_ref_unchecked() }
    }

    #[cpp(body = "return &self->Battery;")]
    fn battery_ptr(&self) -> *mut BrainBattery;

    pub fn battery(&self) -> &BrainBattery {
        unsafe { self.battery_ptr().as_ref_unchecked() }
    }
}

impl Default for &'static Brain {
    fn default() -> Self {
        Brain::get()
    }
}

unsafe impl Send for Brain {}
unsafe impl Sync for Brain {}

opaque!(pub BrainBattery);

impl BrainBattery {
    as_ptr!();

    #[cpp(func = "capacity")]
    pub fn capacity_percent(&self) -> f64;

    #[cpp(func = "temperature")]
    pub fn temperature_percent(&self) -> f64;

    #[cpp(func = "voltage")]
    fn voltage_volts(&self) -> f64;

    pub fn voltage(&self) -> Voltage {
        self.voltage_volts().volts()
    }

    #[cpp(func = "current")]
    fn current_amps(&self) -> f64;

    pub fn current(&self) -> Current {
        self.current_amps().amps()
    }
}

opaque!(pub BrainButton);

impl BrainButton {
    as_ptr!();

    pub fn pressing(&self) -> bool;

    #[cpp(func = "pressed")]
    pub fn on_press(&self, callback: extern "C" fn());

    #[cpp(func = "released")]
    pub fn on_release(&self, callback: extern "C" fn());
}

opaque!(pub BrainLcd);

impl BrainLcd {
    const WIDTH: i32 = 159;
    const HEIGHT: i32 = 107;

    as_ptr!();

    pub fn get() -> &'static Self {
        Brain::get().lcd()
    }

    #[cpp(func = "pressed")]
    fn on_press_raw(&self, callback: extern "C" fn(*mut ()), arg: *mut ());

    pub fn on_press<F: Fn() + Send + Sync + 'static>(&self, f: F) {
        let (callback, arg) = into_callback(f);
        self.on_press_raw(callback, arg);
    }

    #[cpp(func = "released")]
    fn on_release_raw(&self, callback: extern "C" fn(*mut ()), arg: *mut ());

    pub fn on_release<F: Fn() + Send + Sync + 'static>(&self, f: F) {
        let (callback, arg) = into_callback(f);
        self.on_release_raw(callback, arg);
    }

    #[cpp(func = "xPosition")]
    pub fn last_pressed_x(&self) -> i32;

    #[cpp(func = "yPosition")]
    pub fn last_pressed_y(&self) -> i32;

    pub fn pressing(&self) -> bool;

    pub fn lock_with(&'static self, style: &Style) -> BrainLcdGuard {
        BrainLcdGuard::new(self, LCD_LOCK.lock(), style)
    }

    pub fn try_lock_with(&'static self, style: &Style) -> Option<BrainLcdGuard> {
        Some(BrainLcdGuard::new(self, LCD_LOCK.try_lock()?, style))
    }

    // Noted as "not for public use yet" in C++ header
    // pub fn wait_for_refresh(&self);
    // pub fn render_disable(&self);
}

unsafe impl Sync for BrainLcd {}

static LCD_LOCK: Mutex<()> = Mutex::new(());

pub struct BrainLcdGuard {
    lcd: &'static BrainLcd,
    _guard: MutexGuard<'static, ()>,
}

impl BrainLcdGuard {
    pub(crate) fn as_ptr(&self) -> *mut BrainLcd {
        self.lcd.as_ptr()
    }

    fn new(lcd: &'static BrainLcd, guard: MutexGuard<'static, ()>, style: &Style) -> Self {
        let mut result = Self { lcd, _guard: guard };
        result.set_style(style);
        result.set_cursor(0, 0);
        result.set_clip_region(0, 0, BrainLcd::WIDTH, BrainLcd::HEIGHT);
        result
    }

    pub fn set_style(&mut self, style: &Style) {
        self.set_font(style.font);
        self.set_pen_width(style.pen_width);
        self.set_origin(style.origin.0, style.origin.1);
        self.set_pen_color(style.pen_color);
        self.set_fill_color(style.fill_color);
    }

    fn set_font(&self, font: FontType);

    fn set_pen_width(&self, width: u32);

    fn set_origin(&self, x: i32, y: i32);

    fn set_pen_color(&self, color: Color);

    fn set_fill_color(&self, color: Color);

    pub fn set_cursor(&self, row: i32, col: i32);

    pub fn set_clip_region(&self, x: i32, y: i32, width: i32, height: i32);

    #[cpp(func = "column")]
    pub fn col(&self) -> i32;

    pub fn row(&self) -> i32;

    #[cpp(func = "getStringWidth")]
    fn get_string_width_inner(&self, text: *const c_char) -> i32;

    #[cpp(func = "getStringHeight")]
    fn get_string_height_inner(&self, text: *const c_char) -> i32;

    pub fn get_string_width(&self, text: impl CStrArg) -> Result<i32, NulError> {
        Ok(self.get_string_width_inner(text.into_cstr_ptr()?))
    }

    pub fn get_string_height(&self, text: impl CStrArg) -> Result<i32, NulError> {
        Ok(self.get_string_height_inner(text.into_cstr_ptr()?))
    }

    #[cpp(body = r#"self->print("%.*s", (int)text_len, text);"#)]
    pub fn print(&self, text: &str);

    #[cpp(body = r#"self->printAt(x, y, opaque, "%.*s", (int)text_len, text);"#)]
    pub fn print_at(&self, x: i32, y: i32, opaque: bool, text: &str);

    pub fn print_at_opaque(&self, x: i32, y: i32, text: &str) -> Result<(), NulError> {
        self.print_at(x, y, true, text);
        Ok(())
    }

    pub fn print_at_transparent(&self, x: i32, y: i32, text: &str) -> Result<(), NulError> {
        self.print_at(x, y, false, text);
        Ok(())
    }

    #[cpp(func = "clearScreen")]
    pub fn clear(&self);

    #[cpp(func = "clearScreen")]
    pub fn fill(&self, color: Color);

    pub fn clear_line(&self, number: i32);

    #[cpp(func = "clearLine")]
    pub fn clear_current_line(&self);

    #[cpp(func = "clearLine")]
    pub fn fill_line(&self, number: i32, color: Color);

    pub fn new_line(&self);

    pub fn draw_pixel(&self, x: i32, y: i32);

    pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32);

    pub fn draw_rectangle(&self, x: i32, y: i32, width: i32, height: i32);

    #[cpp(func = "drawRectangle")]
    pub fn draw_rectangle_color(&self, x: i32, y: i32, width: i32, height: i32, color: Color);

    pub fn draw_circle(&self, x: i32, y: i32, radius: i32);

    #[cpp(func = "drawCircle")]
    pub fn draw_circle_color(&self, x: i32, y: i32, radius: i32, color: Color);

    #[cpp(func = "drawImageFromBuffer")]
    fn draw_image_inner(&self, buf: *const u8, x: i32, y: i32, buffer_len: i32);

    pub fn draw_image(&self, x: i32, y: i32, buf: &[u8]) {
        self.draw_image_inner(buf.as_ptr(), x, y, buf.len() as i32);
    }

    #[cpp(func = "drawImageFromBuffer")]
    fn draw_buffer_inner(&self, buf: *const u32, x: i32, y: i32, width: i32, height: i32);

    pub fn draw_buffer(&self, x: i32, y: i32, width: i32, height: i32, pixels: &[u32]) {
        self.draw_buffer_inner(pixels.as_ptr(), x, y, width, height);
    }

    #[cpp(func = "drawImageFromFile")]
    fn draw_image_file_inner(&self, name: *const c_char, x: i32, y: i32);

    pub fn draw_image_file(&self, x: i32, y: i32, path: impl CStrArg) -> Result<(), NulError> {
        self.draw_image_file_inner(path.with_cstr(|c| c.as_ptr())?, x, y);
        Ok(())
    }

    #[cpp(func = "render")]
    fn render_inner(&self) -> bool;

    #[cpp(func = "render")]
    fn render_inner_with(&self, vsync_wait: bool, run_scheduler: bool) -> bool;

    pub fn render(&self) -> Result<(), RenderError> {
        if self.render_inner() {
            Ok(())
        } else {
            Err(RenderError)
        }
    }

    pub fn render_vsync(&self, vsync_wait: bool) -> Result<(), RenderError> {
        self.render_with(vsync_wait, true)
    }

    pub fn render_with(&self, vsync_wait: bool, run_scheduler: bool) -> Result<(), RenderError> {
        if self.render_inner_with(vsync_wait, run_scheduler) {
            Ok(())
        } else {
            Err(RenderError)
        }
    }
}
