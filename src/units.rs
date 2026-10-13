use core::{
    f64::consts::PI,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use pastey::paste;

use crate::VelocityUnits;

pub const MM_PER_INCH: f64 = 25.4;
pub const DEG_PER_RAD: f64 = 180.0 / PI;

mod private {
    pub trait Sealed {}
}

macro_rules! number_types {
    ($callback:ident) => {
        $callback!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
    };
}

macro_rules! from_decls {
    ($($ty:ident),+ $(,)?) => {
        $(paste! {
            #[doc(hidden)]
            fn [<from_ $ty>](v: $ty) -> Self;
        })+
    };
}

macro_rules! from_fns {
    ($($ty:ident),+ $(,)?) => {
        $(paste! {
            #[doc(hidden)]
            fn [<from_ $ty>](v: $ty) -> Self {
                v as Self
            }
        })+
    };
}

pub trait Number: private::Sealed + Sized {
    fn cast<T: Number>(self) -> T;

    number_types!(from_decls);
}

macro_rules! impl_number {
    ($($ty:ident),+ $(,)?) => {
        $(
            impl private::Sealed for $ty {}

            impl Number for $ty {
                fn cast<T: Number>(self) -> T {
                    paste! {
                        T::[<from_ $ty>](self)
                    }
                }

                number_types!(from_fns);
            }
        )+
    };
}

number_types!(impl_number);

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Time(f64);

impl Time {
    pub fn ms(self) -> f64 {
        self.0
    }

    pub fn sec(self) -> f64 {
        self.0 / 1000.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Distance(f64);

impl Distance {
    pub fn mm(self) -> f64 {
        self.0
    }

    pub fn inches(self) -> f64 {
        self.0 / MM_PER_INCH
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle(f64);

impl Angle {
    pub fn deg(self) -> f64 {
        self.0
    }

    pub fn rad(self) -> f64 {
        self.0 / DEG_PER_RAD
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Velocity(f64, VelocityUnits);

impl Velocity {
    pub fn value(self) -> f64 {
        self.0
    }

    pub fn units(self) -> VelocityUnits {
        self.1
    }

    pub fn from_value(value: f64, units: VelocityUnits) -> Self {
        Self(value, units)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Voltage(f64);

impl Voltage {
    pub fn volts(self) -> f64 {
        self.0
    }

    pub fn millivolts(self) -> f64 {
        self.0 * 1000.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Torque(f64);

impl Torque {
    pub fn newton_meters(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Current(f64);

impl Current {
    pub fn amps(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f64);

impl Power {
    pub fn watts(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature(f64);

impl Temperature {
    pub fn celsius(self) -> f64 {
        self.0
    }

    pub fn fahrenheit(self) -> f64 {
        self.0 * 1.8 + 32.0
    }
}

macro_rules! impl_math {
    ($($ty:ident),+) => {
        $(
            impl_math!(@traits $ty, Add, Div, Mul, Rem, Sub);
            impl_math!(@assign_traits $ty, AddAssign, DivAssign, MulAssign, RemAssign, SubAssign);

            impl Neg for $ty {
                type Output = Self;

                fn neg(self) -> Self {
                    Self(-self.0)
                }
            }
        )+
    };

    (@traits $ty:ident, $($trait:ident),+ $(,)?) => {
        $(impl $trait for $ty {
            type Output = Self;

            paste! {
                fn [<$trait:snake>](self, rhs: Self) -> Self::Output {
                    Self(self.0.[<$trait:snake>](rhs.0))
                }
            }
        })+
    };

    (@traits $ty:ident, $($trait:ident),+ $(,)?) => {
        $(impl $trait for $ty {
            type Output = Self;

            paste! {
                fn [<$trait:snake>](self, rhs: Self) -> Self::Output {
                    Self(self.0.[<$trait:snake>](rhs.0))
                }
            }
        })+
    };

    (@assign_traits $ty:ident, $($trait:ident),+ $(,)?) => {
        $(impl $trait for $ty {
            paste! {
                fn [<$trait:snake>](&mut self, rhs: Self) {
                    self.0.[<$trait:snake>](rhs.0)
                }
            }
        })+
    };
}

impl_math!(
    Time,
    Distance,
    Angle,
    Voltage,
    Torque,
    Current,
    Power,
    Temperature
);

pub trait Units: private::Sealed {
    fn ms(self) -> Time;
    fn sec(self) -> Time;
    fn mm(self) -> Distance;
    fn inches(self) -> Distance;
    fn deg(self) -> Angle;
    fn rad(self) -> Angle;
    fn percent(self) -> Velocity;
    fn rpm(self) -> Velocity;
    fn volts(self) -> Voltage;
    fn millivolts(self) -> Voltage;
    fn newton_meters(self) -> Torque;
    fn amps(self) -> Current;
    fn watts(self) -> Power;
    fn celsius(self) -> Temperature;
    fn fahrenheit(self) -> Temperature;
}

impl<T: Number> Units for T {
    fn ms(self) -> Time {
        Time(self.cast())
    }

    fn sec(self) -> Time {
        Time(self.cast::<f64>() * 1000.0)
    }

    fn mm(self) -> Distance {
        Distance(self.cast())
    }

    fn inches(self) -> Distance {
        Distance(self.cast::<f64>() * MM_PER_INCH)
    }

    fn deg(self) -> Angle {
        Angle(self.cast())
    }

    fn rad(self) -> Angle {
        Angle(self.cast::<f64>() * DEG_PER_RAD)
    }

    fn percent(self) -> Velocity {
        Velocity(self.cast(), VelocityUnits::Percent)
    }

    fn rpm(self) -> Velocity {
        Velocity(self.cast(), VelocityUnits::Rpm)
    }

    fn volts(self) -> Voltage {
        Voltage(self.cast())
    }

    fn millivolts(self) -> Voltage {
        Voltage(self.cast::<f64>() / 1000.0)
    }

    fn newton_meters(self) -> Torque {
        Torque(self.cast())
    }

    fn amps(self) -> Current {
        Current(self.cast())
    }

    fn watts(self) -> Power {
        Power(self.cast())
    }

    fn celsius(self) -> Temperature {
        Temperature(self.cast())
    }

    fn fahrenheit(self) -> Temperature {
        Temperature((self.cast::<f64>() - 32.0) / 1.8)
    }
}
