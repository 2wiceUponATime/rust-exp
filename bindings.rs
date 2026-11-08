use crate::units::{Time, Units};

use super::*;

use alloc::{
    boxed::Box,
    ffi::{CString, NulError},
    fmt::{self, Write},
    string::{String, ToString},
};
use core::{
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

#[cpp(func = "vexSystemExitRequest")]
pub fn request_exit();

#[cpp(func = "timer::system")]
pub fn time() -> u32;

#[cpp(body = "return fflush(stdout);")]
fn flush_inner() -> i32;

#[cpp(body = r#"printf("%s", value);"#)]
fn print_str(value: CString);

#[cpp(func = "this_thread::sleep_for")]
pub fn sleep(ms: u32);

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let c_string = CString::new(s).map_err(|_| fmt::Error)?;
        print_str(c_string);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = Stdout.write_fmt(args);
}

#[doc(hidden)]
pub fn _format(args: fmt::Arguments) -> String {
    args.to_string()
}

macro_rules! opaque {
    ($($name:ident),+ $(,)?) => {
        $(
            #[repr(C)]
            struct $name {
                _data: [u8; 0],
                _mutable: core::cell::UnsafeCell<[u8; 0]>,
                _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
            }
        )+
    };
}

opaque!(RawBrain, RawBrainBattery, RawBrainButton, RawBrainLcd);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FlushError {
    Unknown(i32),
}

pub fn flush() -> Result<(), FlushError> {
    match flush_inner() {
        0 => Ok(()),
        c => Err(FlushError::Unknown(c)),
    }
}

fn never_null<T>(ptr: *mut T) -> NonNull<T> {
    #[cfg(debug_assertions)]
    return NonNull::new(ptr).expect("unexpected null pointer");
    #[cfg(not(debug_assertions))]
    return unsafe { NonNull::new_unchecked(ptr) };
}

#[repr(u8)]
pub enum FontType {
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

pub struct RenderError;

fn into_callback<F: Fn() + Send + Sync + 'static>(f: F) -> (extern "C" fn(*mut ()), *mut ()) {
    extern "C" fn trampoline<F: Fn()>(arg: *mut ()) {
        let f = unsafe { &*(arg as *const F) };
        f();
    }
    (trampoline::<F>, Box::into_raw(Box::new(f)) as *mut ())
}

static BRAIN: AtomicPtr<RawBrain> = AtomicPtr::new(core::ptr::null_mut());

#[repr(transparent)]
pub struct Brain(RawBrain);

impl Brain {
    fn as_ptr(&self) -> *mut RawBrain {
        self as *const Self as *mut RawBrain
    }

    #[cpp(func = "new brain")]
    fn new_ptr() -> *mut RawBrain;

    #[cpp(body = "delete ptr;")]
    fn free(ptr: *mut RawBrain);

    unsafe fn from_ptr<'a>(ptr: *mut RawBrain) -> &'a Self {
        unsafe { &*(ptr as *mut Self) }
    }

    pub fn get() -> &'static Self {
        let current = BRAIN.load(Ordering::Acquire);
        if !current.is_null() {
            return unsafe { Self::from_ptr(current) };
        }

        let new = Self::new_ptr();

        match BRAIN.compare_exchange(
            core::ptr::null_mut(),
            new,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => unsafe { Self::from_ptr(new) },
            Err(existing) => unsafe {
                Self::free(new);
                Self::from_ptr(existing)
            },
        }
    }

    #[cpp(body = "return &self->buttonUp;")]
    fn button_up_ptr(&self) -> *mut RawBrainButton;

    pub fn button_up(&self) -> &BrainButton {
        unsafe { BrainButton::from_ptr(self.button_up_ptr()) }
    }

    #[cpp(body = "return &self->Screen;")]
    fn lcd_ptr(&self) -> *mut RawBrainLcd;

    pub fn lcd(&self) -> &BrainLcd {
        unsafe { BrainLcd::from_ptr(self.lcd_ptr()) }
    }

    #[cpp(body = "return self->timer(msec);")]
    fn time_ms(&self) -> f64;

    pub fn time(&self) -> Time {
        self.time_ms().ms()
    }

    pub fn reset_timer(&self);

    #[cpp(body = "self->setTimer(value, msec);")]
    pub fn set_timer(&self, value: Time);

    #[cpp(body = "return &self->Battery;")]
    fn battery_ptr(&self) -> *mut RawBrainBattery;

    pub fn battery(&self) -> &BrainBattery {
        unsafe { BrainBattery::from_ptr(self.battery_ptr()) }
    }
}

impl Default for &'static Brain {
    fn default() -> Self {
        Brain::get()
    }
}

unsafe impl Send for Brain {}
unsafe impl Sync for Brain {}

#[repr(transparent)]
pub struct BrainBattery(RawBrainBattery);

impl BrainBattery {
    fn as_ptr(&self) -> *mut RawBrainBattery {
        self as *const Self as *mut RawBrainBattery
    }

    unsafe fn from_ptr<'a>(ptr: *mut RawBrainBattery) -> &'a Self {
        unsafe { &*(ptr as *mut Self) }
    }

    pub fn capacity(&self) -> f64;

    #[cpp(func = "temperature")]
    pub fn temperature_percent(&self) -> f64;
}

#[repr(transparent)]
pub struct BrainButton(RawBrainButton);

impl BrainButton {
    fn as_ptr(&self) -> *mut RawBrainButton {
        self as *const Self as *mut RawBrainButton
    }

    unsafe fn from_ptr<'a>(ptr: *mut RawBrainButton) -> &'a Self {
        unsafe { &*ptr.cast() }
    }

    pub fn pressing(&self) -> bool;

    #[cpp(func = "pressed")]
    pub fn on_press(&self, callback: extern "C" fn());

    #[cpp(func = "released")]
    pub fn on_release(&self, callback: extern "C" fn());
}

#[repr(transparent)]
pub struct BrainLcd(RawBrainLcd);

impl BrainLcd {
    fn as_ptr(&self) -> *mut RawBrainLcd {
        self as *const Self as *mut RawBrainLcd
    }

    unsafe fn from_ptr<'a>(ptr: *mut RawBrainLcd) -> &'a Self {
        unsafe { &*ptr.cast() }
    }

    pub fn get() -> &'static Self {
        Brain::get().lcd()
    }

    pub fn set_cursor(&self, row: i32, col: i32);

    pub fn set_font(&self, font: FontType);

    pub fn set_pen_width(&self, width: u32);

    pub fn set_origin(&self, x: i32, y: i32);

    #[cpp(func = "column")]
    pub fn col(&self) -> i32;

    pub fn row(&self) -> i32;

    pub fn set_fill_color(&self, color: Color);

    #[cpp(func = "setFillColor")]
    fn set_fill_color_web_inner(&self, color: CString);

    pub fn set_fill_color_web(&self, color: &str) -> Result<(), NulError> {
        self.set_fill_color_web_inner(CString::new(color)?);
        Ok(())
    }

    #[cpp(func = "setFillColor")]
    pub fn set_fill_color_hue(&self, hue: i32);

    #[cpp(func = "getStringWidth")]
    fn get_string_width_inner(&self, text: CString) -> i32;

    #[cpp(func = "getStringHeight")]
    fn get_string_height_inner(&self, text: CString) -> i32;

    pub fn get_string_width(&self, text: &str) -> Result<i32, NulError> {
        Ok(self.get_string_width_inner(CString::new(text)?))
    }

    pub fn get_string_height(&self, text: &str) -> Result<i32, NulError> {
        Ok(self.get_string_height_inner(CString::new(text)?))
    }

    #[cpp(body = r#"self->print("%s", text);"#)]
    fn print_inner(&self, text: CString);

    pub fn print(&self, text: &str) -> Result<(), NulError> {
        self.print_inner(CString::new(text)?);
        Ok(())
    }

    #[cpp(body = r#"self->printAt(x, y, opaque, "%s", text);"#)]
    fn print_at_inner(&self, x: i32, y: i32, opaque: bool, text: CString);

    pub fn print_at(&self, x: i32, y: i32, text: &str) -> Result<(), NulError> {
        self.print_at_inner(x, y, true, CString::new(text)?);
        Ok(())
    }

    pub fn print_at_transparent(&self, x: i32, y: i32, text: &str) -> Result<(), NulError> {
        self.print_at_inner(x, y, false, CString::new(text)?);
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

    #[cpp(func = "clearLine")]
    fn fill_line_web_inner(&self, number: i32, color: CString);

    pub fn fill_line_web(&self, number: i32, color: &str) -> Result<(), NulError> {
        self.fill_line_web_inner(number, CString::new(color)?);
        Ok(())
    }

    #[cpp(func = "clearLine")]
    pub fn fill_line_hue(&self, number: i32, hue: i32);

    pub fn new_line(&self);

    pub fn draw_pixel(&self, x: i32, y: i32);

    pub fn draw_line(&self, x1: i32, y1: i32, x2: i32, y2: i32);

    pub fn draw_rectangle(&self, x: i32, y: i32, width: i32, height: i32);

    #[cpp(func = "drawRectangle")]
    pub fn draw_rectangle_color(&self, x: i32, y: i32, width: i32, height: i32, color: Color);

    #[cpp(func = "drawRectangle")]
    fn draw_rectangle_web_inner(&self, x: i32, y: i32, width: i32, height: i32, color: CString);

    pub fn draw_rectangle_web(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        color: &str,
    ) -> Result<(), NulError> {
        self.draw_rectangle_web_inner(x, y, width, height, CString::new(color)?);
        Ok(())
    }

    #[cpp(func = "drawRectangle")]
    pub fn draw_rectangle_hue(&self, x: i32, y: i32, width: i32, height: i32, hue: i32);

    pub fn draw_circle(&self, x: i32, y: i32, radius: i32);

    #[cpp(func = "drawCircle")]
    pub fn draw_circle_color(&self, x: i32, y: i32, radius: i32, color: Color);

    #[cpp(func = "drawCircle")]
    fn draw_circle_web_inner(&self, x: i32, y: i32, radius: i32, color: CString);

    pub fn draw_circle_web(
        &self,
        x: i32,
        y: i32,
        radius: i32,
        color: &str,
    ) -> Result<(), NulError> {
        self.draw_circle_web_inner(x, y, radius, CString::new(color)?);
        Ok(())
    }

    #[cpp(func = "drawCircle")]
    pub fn draw_circle_hue(&self, x: i32, y: i32, radius: i32, hue: i32);

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

    pub fn set_clip_region(&self, x: i32, y: i32, width: i32, height: i32);

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
    fn draw_image_file_inner(&self, name: CString, x: i32, y: i32);

    pub fn draw_image_file(&self, x: i32, y: i32, path: &str) -> Result<(), NulError> {
        self.draw_image_file_inner(CString::new(path)?, x, y);
        Ok(())
    }

    // Noted as "not for public use yet" in C++ header
    // pub fn wait_for_refresh(&self);
    // pub fn render_disable(&self);
}

unsafe impl Send for BrainLcd {}
unsafe impl Sync for BrainLcd {}

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

    #[cpp(body = "return from_vex(to_vex(color).web(value));")]
    fn set_web(color: Color, value: CString) -> Color;

    pub fn web(color: &str) -> Result<Self, NulError> {
        Ok(Self::set_web(Self::default(), CString::new(color)?))
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
