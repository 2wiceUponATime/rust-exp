#![no_std]

extern crate alloc;

use alloc::ffi::{CString, NulError};
use core::{
    fmt::{self, Write},
    marker::PhantomData,
    panic::PanicInfo,
    ptr::NonNull,
};
use ffi::*;

mod allocator;

mod ffi {
    use core::{
        ffi::c_char,
        marker::{PhantomData, PhantomPinned},
    };

    macro_rules! opaque {
        ($($name:ident),+ $(,)?) => {
            $(
                #[repr(C)]
                pub struct $name {
                    _data: [u8; 0],
                    _marker: PhantomData<(*mut u8, PhantomPinned)>,
                }
            )+
        };
    }

    opaque!(RawBrain, RawMotor, RawInertial, RawSmartdrive);

    unsafe extern "C" {
        pub fn exp_printf(s: *const c_char);
        pub fn exp_flush_stdout() -> i32;

        pub fn exp_brain_new() -> *mut RawBrain;
        pub fn exp_brain_free(b: *mut RawBrain);
        pub fn exp_brain_print_at(b: *mut RawBrain, x: i32, y: i32, s: *const c_char);

        pub fn exp_motor_new(port: u8) -> *mut RawMotor;
        pub fn exp_motor_free(m: *mut RawMotor);

        pub fn exp_inertial_new() -> *mut RawInertial;
        pub fn exp_inertial_new_port(port: u8) -> *mut RawInertial;
        pub fn exp_inertial_free(i: *mut RawInertial);

        pub fn exp_smartdrive_new(
            l: *mut RawMotor,
            r: *mut RawMotor,
            i: *mut RawInertial,
        ) -> *mut RawSmartdrive;
        pub fn exp_smartdrive_free(s: *mut RawSmartdrive);
        pub fn exp_smartdrive_turn_to_heading(s: *mut RawSmartdrive, angle: f64);

        pub fn vexSystemExitRequest();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { vexSystemExitRequest() };
    loop {}
}

struct Stdout;

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let c_string = CString::new(s).map_err(|_| fmt::Error)?;
        unsafe {
            exp_printf(c_string.as_ptr());
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    let _ = Stdout.write_fmt(args);
}

pub fn flush() {
    unsafe {
        exp_flush_stdout();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
        $crate::flush();
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
        $crate::flush();
    };
}

pub struct Brain(NonNull<RawBrain>);

impl Brain {
    pub fn new() -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_brain_new())?)) }
    }

    pub fn print_at(&self, x: i32, y: i32, s: &str) -> Result<(), NulError> {
        let c_string = CString::new(s)?;
        unsafe {
            exp_brain_print_at(self.0.as_ptr(), x, y, c_string.as_ptr());
        }
        Ok(())
    }
}

impl Drop for Brain {
    fn drop(&mut self) {
        unsafe {
            exp_brain_free(self.0.as_ptr());
        }
    }
}

pub struct Motor(NonNull<RawMotor>);

impl Motor {
    pub fn new(port: u8) -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_motor_new(port))?)) }
    }
}

impl Drop for Motor {
    fn drop(&mut self) {
        unsafe {
            exp_motor_free(self.0.as_ptr());
        }
    }
}

pub struct Inertial(NonNull<RawInertial>);

impl Inertial {
    pub fn new() -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_inertial_new())?)) }
    }

    pub fn new_port(port: u8) -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_inertial_new_port(port))?)) }
    }
}

impl Drop for Inertial {
    fn drop(&mut self) {
        unsafe {
            exp_inertial_free(self.0.as_ptr());
        }
    }
}

pub struct Smartdrive<'a> {
    raw: NonNull<RawSmartdrive>,
    _deps: PhantomData<(&'a Motor, &'a Motor, &'a Inertial)>,
}

impl<'a> Smartdrive<'a> {
    pub fn new(l: &'a Motor, r: &'a Motor, i: &'a Inertial) -> Option<Self> {
        unsafe {
            Some(Self {
                raw: NonNull::new(exp_smartdrive_new(l.0.as_ptr(), r.0.as_ptr(), i.0.as_ptr()))?,
                _deps: PhantomData,
            })
        }
    }

    pub fn turn_to_heading(&self, angle: f64) {
        unsafe {
            exp_smartdrive_turn_to_heading(self.raw.as_ptr(), angle);
        }
    }
}

impl Drop for Smartdrive<'_> {
    fn drop(&mut self) {
        unsafe {
            exp_smartdrive_free(self.raw.as_ptr());
        }
    }
}
