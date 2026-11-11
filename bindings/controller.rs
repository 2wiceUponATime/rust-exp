use core::{ffi::c_char, ptr::NonNull};

use alloc::ffi::NulError;
use derive_more::TryFrom;

use crate::{
    CStrArg, IntoCStrPtr, as_ptr, opaque,
    thread::{Mutex, MutexGuard},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AxisType {
    Axis3 = 0,
    Axis4,
    Axis1,
    Axis2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ButtonType {
    L1 = 0,
    L2,
    R1,
    R2,
    Up,
    Down,
    Left,
    Right,
    X,
    B,
    Y,
    A,
    L3,
    R3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFrom)]
#[try_from(repr)]
#[repr(u8)]
pub enum ControllerType {
    Primary = 0,
    Partner,
}

static CONTROLLERS: [Mutex<Option<&'static Controller>>; 2] = [Mutex::new(None), Mutex::new(None)];

opaque!(pub Controller);

impl Controller {
    as_ptr!();

    #[cpp(func = "new controller")]
    fn new_ptr(kind: ControllerType) -> *mut Controller;

    pub fn get(kind: ControllerType) -> &'static Self {
        let mut guard = CONTROLLERS[kind as usize].lock();
        guard.get_or_insert_with(|| unsafe {
            NonNull::new(Self::new_ptr(kind))
                .expect("failed to allocate Controller")
                .as_ref()
        })
    }

    pub fn primary() -> &'static Self {
        Self::get(ControllerType::Primary)
    }
    pub fn partner() -> &'static Self {
        Self::get(ControllerType::Partner)
    }

    fn index(&self) -> usize {
        CONTROLLERS
            .iter()
            .position(|slot| slot.lock().is_some_and(|c| core::ptr::eq(c, self)))
            .unwrap()
    }

    pub fn kind(&self) -> ControllerType {
        (self.index() as u8).try_into().unwrap()
    }

    #[cpp(func = "installed")]
    pub fn is_installed(&self) -> bool;

    #[cpp(func = "rumble")]
    fn rumble_inner(&self, pattern: *const c_char);

    pub fn rumble(&self, pattern: impl CStrArg) -> Result<(), NulError> {
        self.rumble_inner(pattern.into_cstr_ptr()?);
        Ok(())
    }

    #[cpp(body = "return get_controller_button(self, kind);")]
    fn button_ptr(&self, kind: ButtonType) -> *const ControllerButton;

    pub fn button(&self, kind: ButtonType) -> &ControllerButton {
        unsafe { self.button_ptr(kind).as_ref_unchecked() }
    }

    #[cpp(body = "return get_controller_axis(self, kind);")]
    fn axis_ptr(&self, kind: AxisType) -> *const ControllerAxis;

    pub fn axis(&self, kind: AxisType) -> &ControllerAxis {
        unsafe { self.axis_ptr(kind).as_ref_unchecked() }
    }

    #[cpp(body = "return &self->Screen;")]
    fn lcd_ptr(&self) -> *mut ControllerLcd;

    pub fn lcd(&self) -> &ControllerLcd {
        unsafe { self.lcd_ptr().as_ref_unchecked() }
    }
}

unsafe impl Sync for Controller {}

opaque!(pub ControllerButton);

impl ControllerButton {
    as_ptr!();

    pub fn pressing(&self) -> bool;

    #[cpp(func = "pressed")]
    pub fn on_press(&self, callback: extern "C" fn());

    #[cpp(func = "released")]
    pub fn on_release(&self, callback: extern "C" fn());
}

opaque!(pub ControllerAxis);

impl ControllerAxis {
    as_ptr!();

    #[cpp(func = "changed")]
    pub fn on_change(&self, callback: extern "C" fn());

    pub fn value(&self) -> i8;
}

opaque!(pub ControllerLcd);

impl ControllerLcd {
    as_ptr!();

    fn index(&self) -> usize {
        CONTROLLERS
            .iter()
            .position(|slot| {
                slot.lock()
                    .is_some_and(|c| core::ptr::eq(c.lcd_ptr(), self))
            })
            .unwrap()
    }

    #[cpp(func = "column")]
    pub fn col(&self);

    pub fn row(&self);

    pub fn lock(&'static self) -> ControllerLcdGuard {
        ControllerLcdGuard {
            lcd: self,
            _lock: LCD_LOCKS[self.index()].lock(),
        }
    }

    pub fn try_lock(&'static self) -> Option<ControllerLcdGuard> {
        Some(ControllerLcdGuard {
            lcd: self,
            _lock: LCD_LOCKS[self.index()].try_lock()?,
        })
    }
}

static LCD_LOCKS: [Mutex<()>; 2] = [Mutex::new(()), Mutex::new(())];

pub struct ControllerLcdGuard {
    lcd: &'static ControllerLcd,
    _lock: MutexGuard<'static, ()>,
}

impl ControllerLcdGuard {
    pub(crate) fn as_ptr(&self) -> *mut ControllerLcd {
        self.lcd.as_ptr()
    }

    pub fn set_cursor(&self, row: i32, col: i32);

    #[cpp(body = r#"self->print("%.*s", (int)text_len, text);"#)]
    pub fn print(&self, text: &str);

    #[cpp(func = "clearScreen")]
    pub fn clear(&self);

    pub fn clear_line(&self, number: i32);

    #[cpp(func = "clearLine")]
    pub fn clear_current_line(&self);

    pub fn new_line(&self);
}
