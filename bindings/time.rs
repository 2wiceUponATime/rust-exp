use core::{
    ops::{Sub, SubAssign},
    time::Duration,
};

use crate::units::Units;

#[cpp(func = "timer::system")]
pub fn time_ms() -> u32;

#[cpp(func = "timer::systemHighResolution")]
pub fn time_us() -> u64;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(u64);

impl Sub for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        (self.0 - rhs.0).us()
    }
}

impl SubAssign for Instant {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Instant {
    pub const PROGRAM_START: Self = Self(0);

    #[inline]
    pub fn now() -> Self {
        Self(time_us())
    }

    #[inline]
    pub fn elapsed(self) -> Duration {
        Self::now() - self
    }

    pub fn abs_diff(self, other: Self) -> Duration {
        self.0.abs_diff(other.0).us()
    }
}
