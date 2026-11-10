use core::{marker::PhantomData, ptr::NonNull, time::Duration};

use crate::{
    Free, Owned, Port, as_ptr, opaque,
    units::{Angle, Current, Power, Temperature, Torque, Units, Velocity, VelocityUnits},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GearSetting {
    R36_1 = 0,
    R18_1,
    R6_1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BrakeType {
    Brake = 0,
    Coast,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Forward = 0,
    Reverse,
}

opaque!(pub Motor);

impl Motor {
    as_ptr!();

    #[cpp(func = "new motor")]
    fn new_ptr(port: Port, reverse: bool) -> *mut Motor;

    pub fn new(port: Port, reverse: bool) -> Owned<Self> {
        unsafe { Owned::from_raw(Self::new_ptr(port, reverse)) }.expect("failed to allocate Motor")
    }

    #[cpp(func = "new motor")]
    fn new_geared_ptr(port: Port, gear_setting: GearSetting, reverse: bool) -> *mut Motor;

    pub fn new_geared(port: Port, gear_setting: GearSetting, reverse: bool) -> Owned<Self> {
        unsafe { Owned::from_raw(Self::new_geared_ptr(port, gear_setting, reverse)) }
            .expect("failed to allocate Motor")
    }

    #[cpp(func = "installed")]
    pub fn is_installed(&self) -> bool;

    pub fn set_reversed(&self, reverse: bool);

    pub fn reset_position(&self);

    #[cpp(body = "self->setPosition(value, deg);")]
    pub fn set_position(&self, value: Angle);

    #[cpp(body = "self->setTimeout(value, msec);")]
    pub fn set_timeout(&self, value: Duration);

    pub fn spin(&self, dir: Direction, velocity: Velocity);

    #[cpp(body = "self->spinToPosition(angle, deg, velocity_value, to_vex(velocity_units), wait);")]
    pub fn spin_to(&self, angle: Angle, velocity: Velocity, wait: bool);

    #[cpp(
        body = "self->spinFor(to_vex(dir), angle, deg, velocity_value, to_vex(velocity_units), wait);"
    )]
    pub fn spin_for(&self, dir: Direction, angle: Angle, velocity: Velocity, wait: bool);

    pub fn is_spinning(&self) -> bool;

    pub fn is_done(&self) -> bool;

    pub fn stop(&self, mode: BrakeType);

    #[cpp(body = "self->setMaxTorque(value, Nm);")]
    pub fn set_max_torque(&self, value: Torque);

    #[cpp(body = "self->setMaxTorque(value, pct);")]
    pub fn set_max_torque_percent(&self, value: f64);

    #[cpp(body = "self->setMaxTorque(value, amp);")]
    pub fn set_max_torque_current(&self, value: Current);

    #[cpp(body = "return static_cast<uint8_t>(self->direction());")]
    fn direction_inner(&self) -> u8;

    pub fn direction(&self) -> Option<Direction> {
        const FORWARD: u8 = Direction::Forward as u8;
        const REVERSE: u8 = Direction::Reverse as u8;

        match self.direction_inner() {
            FORWARD => Some(Direction::Forward),
            REVERSE => Some(Direction::Reverse),
            _ => None,
        }
    }

    #[cpp(body = "return self->position(deg);")]
    fn position_deg(&self) -> f64;

    pub fn position(&self) -> Angle {
        self.position_deg().deg()
    }

    pub fn velocity(&self, units: VelocityUnits) -> f64;

    #[cpp(func = "current")]
    fn current_amps(&self) -> f64;

    pub fn current(&self) -> Current {
        self.current_amps().amps()
    }

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
    pub fn efficiency_percent(&self);

    #[cpp(body = "return self->temperature(celsius);")]
    fn temperature_celsius(&self) -> f64;

    pub fn temperature(&self) -> Temperature {
        self.temperature_celsius().celsius()
    }

    #[cpp(func = "temperature")]
    pub fn temperature_percent(&self);

    pub fn convert_velocity(&self, value: Velocity, units: VelocityUnits) -> f64;

    #[cpp(func = "getMotorCartridge")]
    pub fn cartridge(&self) -> GearSetting;
}

impl Free for Motor {
    #[cpp(body = "delete ptr;")]
    unsafe fn free(ptr: NonNull<Motor>);
}

opaque!(RawMotorGroup);

#[repr(C)]
pub struct MotorGroup<'a> {
    _raw: RawMotorGroup,
    _motors: PhantomData<&'a Motor>,
}

impl<'a> MotorGroup<'a> {
    as_ptr!();

    #[cpp(body = "return group_from_motors(motors, motor_count);")]
    fn new_ptr(motors: *const *const Motor, motor_count: usize) -> *mut RawMotorGroup;

    pub fn new(motors: &[&'a Motor]) -> Owned<Self> {
        let ptr = Self::new_ptr(motors.as_ptr().cast(), motors.len()).cast();
        unsafe { Owned::from_raw(ptr) }.expect("failed to allocate MotorGroup")
    }

    #[cpp(func = "count")]
    fn count_inner(&self) -> i32;

    pub fn count(&self) -> usize {
        self.count_inner()
            .try_into()
            .expect("motor_group returned a negative count")
    }

    pub fn reset_position(&self);

    #[cpp(body = "self->setPosition(value, deg);")]
    pub fn set_position(&self, value: Angle);

    #[cpp(body = "self->setTimeout(value, msec);")]
    pub fn set_timeout(&self, value: Duration);

    pub fn spin(&self, dir: Direction, velocity: Velocity);

    #[cpp(body = "self->spinToPosition(angle, deg, velocity_value, to_vex(velocity_units), wait);")]
    pub fn spin_to(&self, angle: Angle, velocity: Velocity, wait: bool);

    #[cpp(
        body = "self->spinFor(to_vex(dir), angle, deg, velocity_value, to_vex(velocity_units), wait);"
    )]
    pub fn spin_for(&self, dir: Direction, angle: Angle, velocity: Velocity, wait: bool);

    pub fn is_spinning(&self) -> bool;

    pub fn is_done(&self) -> bool;

    pub fn stop(&self, mode: BrakeType);

    #[cpp(body = "self->setMaxTorque(value, Nm);")]
    pub fn set_max_torque(&self, value: Torque);

    #[cpp(body = "self->setMaxTorque(value, pct);")]
    pub fn set_max_torque_percent(&self, value: f64);

    #[cpp(body = "self->setMaxTorque(value, amp);")]
    pub fn set_max_torque_current(&self, value: Current);

    #[cpp(body = "return static_cast<uint8_t>(self->direction());")]
    fn direction_inner(&self) -> u8;

    pub fn direction(&self) -> Option<Direction> {
        const FORWARD: u8 = Direction::Forward as u8;
        const REVERSE: u8 = Direction::Reverse as u8;

        match self.direction_inner() {
            FORWARD => Some(Direction::Forward),
            REVERSE => Some(Direction::Reverse),
            _ => None,
        }
    }

    #[cpp(body = "return self->position(deg);")]
    fn position_deg(&self) -> f64;

    pub fn position(&self) -> Angle {
        self.position_deg().deg()
    }

    pub fn velocity(&self, units: VelocityUnits) -> f64;

    #[cpp(func = "current")]
    fn current_amps(&self) -> f64;

    pub fn current(&self) -> Current {
        self.current_amps().amps()
    }

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
    pub fn efficiency_percent(&self);

    #[cpp(body = "return self->temperature(celsius);")]
    fn temperature_celsius(&self) -> f64;

    pub fn temperature(&self) -> Temperature {
        self.temperature_celsius().celsius()
    }

    #[cpp(func = "temperature")]
    pub fn temperature_percent(&self);

    pub fn convert_velocity(&self, value: Velocity, units: VelocityUnits) -> f64;

    #[cpp(func = "getMotorCartridge")]
    pub fn cartridge(&self) -> GearSetting;
}

impl Free for MotorGroup<'_> {
    #[cpp(body = "delete ptr;")]
    unsafe fn free(ptr: NonNull<MotorGroup>);
}
