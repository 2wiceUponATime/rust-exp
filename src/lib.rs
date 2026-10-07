#![no_std]

extern crate alloc;

use alloc::{
    ffi::{CString, NulError},
    string::{String, ToString},
};
use core::{
    fmt::{self, Write},
    marker::PhantomData,
    panic::PanicInfo,
    ptr::NonNull,
};
use ffi::*;

mod allocator;

mod ffi {
    use super::*;
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

        pub fn exp_thread_sleep(time: u32);

        pub fn exp_brain_new() -> *mut RawBrain;
        pub fn exp_brain_free(b: *mut RawBrain);
        pub fn exp_brain_screen_clear(b: *mut RawBrain);
        pub fn exp_brain_screen_print_at(b: *mut RawBrain, x: i32, y: i32, s: *const c_char);

        pub fn exp_motor_new(port: u8, reverse: bool) -> *mut RawMotor;
        pub fn exp_motor_free(m: *mut RawMotor);

        pub fn exp_inertial_new() -> *mut RawInertial;
        pub fn exp_inertial_new_port(port: u8) -> *mut RawInertial;
        pub fn exp_inertial_free(i: *mut RawInertial);
        pub fn exp_inertial_calibrate(i: *mut RawInertial);
        pub fn exp_inertial_is_calibrating(i: *mut RawInertial) -> bool;
        pub fn exp_inertial_angle(i: *mut RawInertial) -> f64;

        pub fn exp_smartdrive_new(
            l: *mut RawMotor,
            r: *mut RawMotor,
            i: *mut RawInertial,
        ) -> *mut RawSmartdrive;
        pub fn exp_smartdrive_free(s: *mut RawSmartdrive);
        pub fn exp_smartdrive_set_drive_velocity(s: *mut RawSmartdrive, velocity: f64);
        pub fn exp_smartdrive_set_turn_velocity(s: *mut RawSmartdrive, velocity: f64);
        pub fn exp_smartdrive_turn_to_heading(s: *mut RawSmartdrive, angle: f64);
        pub fn exp_smartdrive_drive(s: *mut RawSmartdrive, dir: DriveDirection);
        pub fn exp_smartdrive_turn(s: *mut RawSmartdrive, dir: TurnDirection);
        pub fn exp_smartdrive_stop(s: *mut RawSmartdrive);

        pub fn vexSystemExitRequest();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    let _ = flush();
    unsafe { vexSystemExitRequest() };
    loop {
        sleep(1000);
    }
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

#[doc(hidden)]
pub fn _format(args: fmt::Arguments) -> String {
    args.to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlushError {
    Unknown(i32),
}

pub fn flush() -> Result<(), FlushError> {
    match unsafe { exp_flush_stdout() } {
        0 => Ok(()),
        c => Err(FlushError::Unknown(c)),
    }
}

pub fn sleep(ms: u32) {
    unsafe {
        exp_thread_sleep(ms);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! screen_print_at {
    ($brain:expr, $x:expr, $y:expr, $($arg:tt)*) => {
        $brain.screen_print_at($x, $y, &$crate::_format(format_args!($($arg)*)))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
        let _ = $crate::flush();
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
        let _ = $crate::flush();
    };
}

pub struct Brain(NonNull<RawBrain>);

impl Brain {
    pub fn new() -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_brain_new())?)) }
    }

    pub fn screen_clear(&self) {
        unsafe {
            exp_brain_screen_clear(self.0.as_ptr());
        }
    }

    pub fn screen_print_at(&self, x: i32, y: i32, s: &str) -> Result<(), NulError> {
        let c_string = CString::new(s)?;
        unsafe {
            exp_brain_screen_print_at(self.0.as_ptr(), x, y, c_string.as_ptr());
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

#[repr(u8)]
pub enum DriveDirection {
    Forward = 0,
    Reverse = 1,
}

#[repr(u8)]
pub enum TurnDirection {
    Left = 0,
    Right = 1,
}

pub struct Motor(NonNull<RawMotor>);

impl Motor {
    pub fn new(port: u8, reverse: bool) -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_motor_new(port, reverse))?)) }
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

    pub fn calibrate(&self) {
        unsafe {
            exp_inertial_calibrate(self.0.as_ptr());
        }
    }

    pub fn is_calibrating(&self) -> bool {
        unsafe { exp_inertial_is_calibrating(self.0.as_ptr()) }
    }

    pub fn angle(&self) -> f64 {
        unsafe { exp_inertial_angle(self.0.as_ptr()) }
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

    pub fn set_drive_velocity(&self, velocity: f64) {
        unsafe {
            exp_smartdrive_set_drive_velocity(self.raw.as_ptr(), velocity);
        }
    }

    pub fn set_turn_velocity(&self, velocity: f64) {
        unsafe {
            exp_smartdrive_set_turn_velocity(self.raw.as_ptr(), velocity);
        }
    }

    pub fn turn_to_heading(&self, angle: f64) {
        unsafe {
            exp_smartdrive_turn_to_heading(self.raw.as_ptr(), angle);
        }
    }

    pub fn drive(&self, dir: DriveDirection) {
        unsafe {
            exp_smartdrive_drive(self.raw.as_ptr(), dir);
        }
    }

    pub fn turn(&self, dir: TurnDirection) {
        unsafe {
            exp_smartdrive_turn(self.raw.as_ptr(), dir);
        }
    }

    pub fn stop(&self) {
        unsafe {
            exp_smartdrive_stop(self.raw.as_ptr());
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
