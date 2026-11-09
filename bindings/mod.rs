use super::*;

use alloc::{
    boxed::Box,
    ffi::{CString, NulError},
    string::String,
};
use core::ffi::{CStr, c_char};

pub mod brain;
pub mod fs;
pub mod io;
pub mod thread;
pub mod time;

#[cpp(func = "vexSystemExitRequest")]
pub fn request_exit();

pub trait CStrArg {
    fn with_cstr<R>(self, f: impl FnOnce(&CStr) -> R) -> Result<R, NulError>;
}

impl CStrArg for &CStr {
    fn with_cstr<R>(self, f: impl FnOnce(&CStr) -> R) -> Result<R, NulError> {
        Ok(f(self))
    }
}

impl CStrArg for &str {
    fn with_cstr<R>(self, f: impl FnOnce(&CStr) -> R) -> Result<R, NulError> {
        Ok(f(&CString::new(self)?))
    }
}

impl CStrArg for String {
    fn with_cstr<R>(self, f: impl FnOnce(&CStr) -> R) -> Result<R, NulError> {
        Ok(f(&CString::new(self)?))
    }
}

pub trait AsCStrPtr: CStrArg + Sized {
    fn as_cstr_ptr(self) -> Result<*const c_char, NulError> {
        self.with_cstr(|c| c.as_ptr())
    }
}

impl<T: CStrArg> AsCStrPtr for T {}

macro_rules! opaque {
    ($($vis:vis $name:ident),+ $(,)?) => {
        $(
            #[repr(C)]
            $vis struct $name {
                _data: [u8; 0],
                _mutable: core::cell::UnsafeCell<[u8; 0]>,
                _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
            }
        )+
    };
}

use opaque;

fn into_callback<F: Fn() + Send + Sync + 'static>(f: F) -> (extern "C" fn(*mut ()), *mut ()) {
    extern "C" fn trampoline<F: Fn()>(arg: *mut ()) {
        let f = unsafe { &*(arg as *const F) };
        f();
    }
    (trampoline::<F>, Box::into_raw(Box::new(f)) as *mut ())
}
