use core::{f64::consts::PI, time::Duration};

pub const MM_PER_INCH: f64 = 25.4;
pub const DEG_PER_RAD: f64 = 180.0 / PI;

mod sealed {
    pub trait Sealed {}
}

macro_rules! impl_debug {
    ($ty:ident, $fmt:literal, $method:ident) => {
        impl core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, concat!(stringify!($ty), "(", $fmt, ")"), self.$method())
            }
        }
    };
}

macro_rules! derive {
    (@derive_more ($($t:ident),+ $(,)?) $item:item) => {
        #[derive(Default, Clone, Copy, PartialEq, PartialOrd, $(derive_more::$t),+)]
        $item
    };

    { $item:item } => {
        derive!(@derive_more (
            Add,
            AddAssign,
            Sub,
            SubAssign,
            Mul,
            MulAssign,
            Div,
            DivAssign,
            Rem,
            RemAssign,
            Neg,
        ) $item);
    };
}

derive! {
    pub struct Distance(f64);
}

impl_debug!(Distance, "{} mm", mm);

impl Distance {
    pub fn mm(self) -> f64 {
        self.0
    }

    pub fn inches(self) -> f64 {
        self.0 / MM_PER_INCH
    }
}

derive! {
    pub struct Angle(f64);
}

impl_debug!(Angle, "{} deg", deg);

impl Angle {
    pub fn deg(self) -> f64 {
        self.0
    }

    pub fn rad(self) -> f64 {
        self.0 / DEG_PER_RAD
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VelocityUnits {
    Percent = 0,
    Rpm,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

derive! {
    pub struct Voltage(f64);
}

impl_debug!(Voltage, "{} V", volts);

impl Voltage {
    pub fn volts(self) -> f64 {
        self.0
    }

    pub fn millivolts(self) -> f64 {
        self.0 * 1000.0
    }
}

derive! {
    pub struct Torque(f64);
}

impl_debug!(Torque, "{} Nm", newton_meters);

impl Torque {
    pub fn newton_meters(self) -> f64 {
        self.0
    }
}

derive! {
    pub struct Current(f64);
}

impl_debug!(Current, "{} A", amps);

impl Current {
    pub fn amps(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Power(f64);

impl_debug!(Power, "{} W", watts);

impl Power {
    pub fn watts(self) -> f64 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature(f64);

impl_debug!(Temperature, "{} C", celsius);

impl Temperature {
    pub fn celsius(self) -> f64 {
        self.0
    }

    pub fn fahrenheit(self) -> f64 {
        self.0 * 1.8 + 32.0
    }
}

pub trait Units: sealed::Sealed {
    fn ms(self) -> Duration;
    fn us(self) -> Duration;
    fn sec(self) -> Duration;
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

macro_rules! float_unit_fns {
    () => {
        fn mm(self) -> Distance {
            Distance(self as f64)
        }
        fn inches(self) -> Distance {
            Distance(self as f64 * MM_PER_INCH)
        }
        fn deg(self) -> Angle {
            Angle(self as f64)
        }
        fn rad(self) -> Angle {
            Angle(self as f64 * DEG_PER_RAD)
        }
        fn percent(self) -> Velocity {
            Velocity(self as f64, VelocityUnits::Percent)
        }
        fn rpm(self) -> Velocity {
            Velocity(self as f64, VelocityUnits::Rpm)
        }
        fn volts(self) -> Voltage {
            Voltage(self as f64)
        }
        fn millivolts(self) -> Voltage {
            Voltage(self as f64 / 1000.0)
        }
        fn newton_meters(self) -> Torque {
            Torque(self as f64)
        }
        fn amps(self) -> Current {
            Current(self as f64)
        }
        fn watts(self) -> Power {
            Power(self as f64)
        }
        fn celsius(self) -> Temperature {
            Temperature(self as f64)
        }
        fn fahrenheit(self) -> Temperature {
            Temperature((self as f64 - 32.0) / 1.8)
        }
    };
}
macro_rules! impl_units_int {
    ($($t:ty),+ $(,)?) => { $(
        impl sealed::Sealed for $t {}
        impl Units for $t {
            fn ms(self) -> Duration {
                Duration::from_millis(self.try_into().unwrap_or(0))
            }
            fn us(self) -> Duration {
                Duration::from_micros(self.try_into().unwrap_or(0))
            }
            fn sec(self) -> Duration {
                Duration::from_secs(self.try_into().unwrap_or(0))
            }
            float_unit_fns!();
        }
    )+ };
}
macro_rules! impl_units_float {
    ($($t:ty),+ $(,)?) => { $(
        impl sealed::Sealed for $t {}
        impl Units for $t {
            fn ms(self) -> Duration {
                Duration::from_secs_f64(self as f64 / 1000.0)
            }
            fn us(self) -> Duration {
                Duration::from_secs_f64(self as f64 / 1_000_000.0)
            }
            fn sec(self) -> Duration {
                Duration::from_secs_f64(self as f64)
            }
            float_unit_fns!();
        }
    )+ };
}

impl_units_int!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
);
impl_units_float!(f32, f64);
