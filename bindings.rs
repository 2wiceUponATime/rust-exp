use super::*;

use alloc::{
    ffi::CString,
    fmt::{self, Write},
    string::{String, ToString},
};
use core::ptr::NonNull;

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
                _marker: PhantomData<(*mut u8, PhantomPinned)>,
            }
        )+
    };
}

opaque!(RawBrain, RawBrainButton, RawBrainLcd);

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

pub struct Brain(NonNull<RawBrain>);

impl Brain {
    #[cpp(func = "new brain")]
    fn new_ptr() -> *mut RawBrain;

    pub fn new() -> Self {
        // SAFETY: `new` throws or aborts on allocation failure; it never returns null.
        Self(never_null(Self::new_ptr()))
    }

    #[cpp(body = "return &self->Screen;")]
    fn lcd_ptr(&self) -> *mut RawBrainLcd;

    #[cpp(body = "delete self;")]
    fn free(&self);

    pub fn lcd<'a>(&'a self) -> BrainLcd<'a> {
        // SAFETY: every non-null `brain` has a non-null address for its `Screen`
        BrainLcd(never_null(self.lcd_ptr()), PhantomData)
    }
}

impl Drop for Brain {
    fn drop(&mut self) {
        self.free();
    }
}

pub struct BrainButton(NonNull<RawBrainButton>);

impl BrainButton {
    #[cpp(func = "new brain::button")]
    fn new_ptr(id: i32) -> *mut RawBrainButton;

    #[cpp(body = "delete self;")]
    fn free(&self);

    pub fn new(id: i32) -> Self {
        Self(never_null(Self::new_ptr(id)))
    }

    pub fn up() -> Self {
        Self::new(0)
    }

    pub fn down() -> Self {
        Self::new(1)
    }

    pub fn check() -> Self {
        Self::new(2)
    }

    pub fn pressing(&self) -> bool;

    #[cpp(func = "pressed")]
    pub fn on_press(&self, callback: extern "C" fn());

    #[cpp(func = "released")]
    pub fn on_release(&self, callback: extern "C" fn());
}

impl Drop for BrainButton {
    fn drop(&mut self) {
        self.free();
    }
}

pub struct BrainLcd<'a>(NonNull<RawBrainLcd>, PhantomData<&'a Brain>);

impl BrainLcd<'_> {
    #[cpp(func = "clearScreen")]
    pub fn clear(&self);
}
