use core::{marker::PhantomData, ptr::NonNull};

use crate::{
    Free, Owned, as_ptr,
    motor::{BrakeType, Direction, Motor, MotorGroup, RawMotorGroup},
    opaque,
    units::{Angle, Current, Distance, Power, Torque, Units, Velocity, VelocityUnits},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TurnDirection {
    Left = 0,
    Right,
}

opaque!(RawDrivetrain);

#[repr(C)]
pub struct Drivetrain<'a> {
    _raw: RawDrivetrain,
    _motors: PhantomData<&'a Motor>,
}

impl Free for Drivetrain<'_> {
    #[cpp(body = "delete ptr;")]
    unsafe fn free(ptr: NonNull<Drivetrain>);
}

impl<'a> Drivetrain<'a> {
    as_ptr!();

    #[cpp(
        body = "return new drivetrain(*left, *right, wheel_travel, track_width, wheel_base, mm, external_gear_ratio);"
    )]
    fn new_ptr(
        left: *mut Motor,
        right: *mut Motor,
        wheel_travel: Distance,
        track_width: Distance,
        wheel_base: Distance,
        external_gear_ratio: f64,
    ) -> *mut RawDrivetrain;

    pub fn new(
        left: &'a Motor,
        right: &'a Motor,
        wheel_travel: Distance,
        track_width: Distance,
        wheel_base: Distance,
        external_gear_ratio: f64,
    ) -> Owned<Self> {
        unsafe {
            Owned::from_raw(
                Self::new_ptr(
                    left.as_ptr(),
                    right.as_ptr(),
                    wheel_travel,
                    track_width,
                    wheel_base,
                    external_gear_ratio,
                )
                .cast(),
            )
        }
        .expect("failed to allocate Drivetrain")
    }

    #[cpp(
        body = "return new drivetrain(*left, *right, wheel_travel, track_width, wheel_base, mm, external_gear_ratio);"
    )]
    fn new_groups_ptr(
        left: *mut RawMotorGroup,
        right: *mut RawMotorGroup,
        wheel_travel: Distance,
        track_width: Distance,
        wheel_base: Distance,
        external_gear_ratio: f64,
    ) -> *mut RawDrivetrain;

    pub fn new_groups(
        left: &MotorGroup<'a>,
        right: &MotorGroup<'a>,
        wheel_travel: Distance,
        track_width: Distance,
        wheel_base: Distance,
        external_gear_ratio: f64,
    ) -> Owned<Self> {
        unsafe {
            Owned::from_raw(
                Self::new_groups_ptr(
                    left.as_ptr() as *mut RawMotorGroup,
                    right.as_ptr() as *mut RawMotorGroup,
                    wheel_travel,
                    track_width,
                    wheel_base,
                    external_gear_ratio,
                )
                .cast(),
            )
        }
        .expect("failed to allocate Drivetrain")
    }

    pub fn drive(&self, dir: Direction, velocity: Velocity);

    #[cpp(
        body = "self->driveFor(to_vex(dir), distance, mm, velocity, to_vex(velocity_units), wait);"
    )]
    pub fn drive_for(&self, dir: Direction, distance: Distance, velocity: Velocity, wait: bool);

    pub fn turn(&self, dir: TurnDirection, velocity: Velocity);

    #[cpp(
        body = "self->turnFor(to_vex(dir), angle, degrees, velocity, to_vex(velocity_units), wait);"
    )]
    pub fn turn_for(&self, dir: TurnDirection, angle: Angle, velocity: Velocity, wait: bool);

    pub fn is_moving(&self) -> bool;

    pub fn is_done(&self) -> bool;

    pub fn stop(&self, mode: BrakeType);

    pub fn velocity(&self, units: VelocityUnits);

    #[cpp(func = "current")]
    fn current_amps(&self) -> f64;

    pub fn current(&self) -> Current {
        self.current_amps().amps()
    }

    #[cpp(body = "return self->current(pct);")]
    pub fn current_percent(&self) -> f64;

    #[cpp(func = "power")]
    fn power_watts(&self) -> f64;

    pub fn power(&self) -> Power {
        self.power_watts().watts()
    }

    #[cpp(func = "torque")]
    fn torque_newton_meters(&self) -> f64;

    pub fn torque(&self) -> Torque {
        self.torque_newton_meters().newton_meters()
    }

    #[cpp(func = "efficiency")]
    pub fn efficiency_percent(&self) -> f64;

    #[cpp(body = "return self->temperature(pct);")]
    pub fn temperature_percent(&self) -> f64;
}

pub struct DrivetrainBuilder {
    wheel_travel: Distance,
    track_width: Distance,
    wheel_base: Distance,
    external_gear_ratio: f64,
}

impl Default for DrivetrainBuilder {
    fn default() -> Self {
        Self {
            wheel_travel: 320.mm(),
            track_width: 320.mm(),
            wheel_base: 130.mm(),
            external_gear_ratio: 1.0,
        }
    }
}

impl DrivetrainBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build<'a>(&self, left: &'a Motor, right: &'a Motor) -> Owned<Drivetrain<'a>> {
        Drivetrain::new(
            left,
            right,
            self.wheel_travel,
            self.track_width,
            self.wheel_base,
            self.external_gear_ratio,
        )
    }

    pub fn build_groups<'a>(
        &self,
        left: &MotorGroup<'a>,
        right: &MotorGroup<'a>,
    ) -> Owned<Drivetrain<'a>> {
        Drivetrain::new_groups(
            left,
            right,
            self.wheel_travel,
            self.track_width,
            self.wheel_base,
            self.external_gear_ratio,
        )
    }

    pub fn wheel_travel(mut self, wheel_travel: Distance) -> Self {
        self.wheel_travel = wheel_travel;
        self
    }

    pub fn track_width(mut self, track_width: Distance) -> Self {
        self.track_width = track_width;
        self
    }

    pub fn wheel_base(mut self, wheel_base: Distance) -> Self {
        self.wheel_base = wheel_base;
        self
    }

    pub fn external_gear_ratio(mut self, external_gear_ratio: f64) -> Self {
        self.external_gear_ratio = external_gear_ratio;
        self
    }
}
