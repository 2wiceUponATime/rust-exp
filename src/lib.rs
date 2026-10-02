#![no_std]

extern crate alloc;

use alloc::ffi::CString;
use core::{panic::PanicInfo, ptr::NonNull};
use ffi::*;

mod allocator;

mod ffi {
    use core::{
        ffi::c_char,
        marker::{PhantomData, PhantomPinned},
    };

    macro_rules! opaque {
        ($name:ident) => {
            #[repr(C)]
            pub struct $name {
                _data: [u8; 0],
                _marker: PhantomData<(*mut u8, PhantomPinned)>,
            }
        };
    }

    opaque!(RawBrain);

    unsafe extern "C" {
        pub fn exp_brain_new() -> *mut RawBrain;
        pub fn exp_brain_free(b: *mut RawBrain);
        pub fn exp_brain_print_at(b: *mut RawBrain, x: i32, y: i32, text: *const c_char);
        pub fn vexSystemExitRequest();
    }
}

pub struct Brain(NonNull<RawBrain>);

impl Brain {
    pub fn new() -> Self {
        unsafe { Self(NonNull::new(exp_brain_new()).unwrap()) }
    }

    pub fn print_at(&mut self, x: i32, y: i32, text: &str) {
        let c_string = CString::new(text).unwrap_or_else(|_| CString::new("").unwrap());
        unsafe {
            exp_brain_print_at(self.0.as_ptr(), x, y, c_string.as_ptr());
        }
    }
}

impl Default for Brain {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Brain {
    fn drop(&mut self) {
        unsafe {
            exp_brain_free(self.0.as_ptr());
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { vexSystemExitRequest() };
    loop {}
}
