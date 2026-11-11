use alloc::{
    boxed::Box,
    ffi::{CString, NulError},
    string::String,
};
use core::{
    ffi::{CStr, c_char},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
};
use derive_more::TryFrom;

pub mod brain;
pub mod controller;
pub mod drivetrain;
pub mod fs;
pub mod io;
pub mod motor;
pub mod thread;
pub mod time;

#[cpp(func = "vexSystemExitRequest")]
pub fn request_exit();

macro_rules! opaque {
    ($vis:vis $name:ident) => {
        #[repr(C)]
        $vis struct $name {
            _data: [u8; 0],
            _mutable: core::cell::UnsafeCell<[u8; 0]>,
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }
    };
}

macro_rules! variant {
    ($vis:vis enum $name:ident {
        $($variant:ident $(($value:ty))?),+ $(,)?
    }) => {
        $vis enum $name {
            $($variant $(($value))?),+
        }

        $($(impl From<$value> for $name {
            fn from(value: $value) -> Self {
                Self::$variant(value)
            }
        })?)+
    };
}

macro_rules! as_ptr {
    () => {
        pub(crate) fn as_ptr(&self) -> *mut Self {
            self as *const Self as *mut Self
        }
    };
}

pub(crate) use {as_ptr, opaque, variant};

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

pub trait IntoCStrPtr: CStrArg + Sized {
    fn into_cstr_ptr(self) -> Result<*const c_char, NulError> {
        self.with_cstr(|c| c.as_ptr())
    }
}

impl<T: CStrArg> IntoCStrPtr for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum Port {
    P1 = 1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
    P8,
    P9,
    P10,
    P11,
    P12,
    P13,
    P14,
    P15,
    P16,
    P17,
    P18,
    P19,
    P20,
    P21,
    P22,
}

impl Port {
    pub fn index(self) -> u8 {
        self as u8 - 1
    }
}

pub(crate) fn into_callback<F: Fn() + Send + Sync + 'static>(
    f: F,
) -> (extern "C" fn(*mut ()), *mut ()) {
    extern "C" fn trampoline<F: Fn()>(arg: *mut ()) {
        let f = unsafe { &*(arg as *const F) };
        f();
    }
    (trampoline::<F>, Box::into_raw(Box::new(f)) as *mut ())
}

pub trait Free {
    /// # Safety
    /// - `ptr` must point to a live `T` that was produced by a constructor
    ///   whose matching release function is this one.
    /// - The caller must own `ptr`. After this call, `ptr` is dangling.
    /// - `ptr` must not have been passed to `free` before.
    unsafe fn free(ptr: NonNull<Self>);
}

pub struct Owned<T: Free>(NonNull<T>, PhantomData<T>);

impl<T: Free> Owned<T> {
    /// # Safety
    /// - Unless null, `ptr` must point to a unique `T` that is live for the
    ///   entire lifetime of the returned `Owned`.
    /// - `ptr` must satisfy the safety requirements of [`Free::free`], since
    ///   it will be pased to `T::free` when the `Owned` is dropped.
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|p| Self(p, PhantomData))
    }

    /// # Safety
    /// - `ptr` must point to a unique `T` that is live for the entire lifetime
    ///   of the returned `Owned`.
    /// - `ptr` must satisfy the safety requirements of [`Free::free`], since
    ///   it will be pased to `T::free` when the `Owned` is dropped.
    pub unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        Self(ptr, PhantomData)
    }

    pub fn into_raw(self) -> *mut T {
        ManuallyDrop::new(self).0.as_ptr()
    }

    pub fn leak(self) -> &'static T {
        unsafe { self.into_raw().as_ref_unchecked() }
    }
}

impl<T: Free> Deref for Owned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T: Free> Drop for Owned<T> {
    fn drop(&mut self) {
        unsafe {
            T::free(self.0);
        }
    }
}

unsafe impl<T: Free + Send> Send for Owned<T> {}
unsafe impl<T: Free + Sync> Sync for Owned<T> {}
