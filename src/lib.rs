#![no_std]

extern crate alloc;

use alloc::{
    ffi::{CString, NulError},
    string::{String, ToString},
    vec::Vec,
};
use core::{
    fmt::{self, Write},
    marker::PhantomData,
    panic::PanicInfo,
    ptr::NonNull,
};
use ffi::*;
use libm::round;
use units::*;

pub mod units;

mod allocator;
mod ffi;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    let _ = flush();
    unsafe {
        exp_request_exit();
    }
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

pub fn get_time() -> u32 {
    unsafe { exp_time() }
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Direction {
    Forward = 0,
    Reverse = 1,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TurnDirection {
    Left = 0,
    Right = 1,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BrakeType {
    Coast = 0,
    Brake = 1,
    Hold = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VelocityUnits {
    Percent = 0,
    Rpm = 1,
}

pub struct Drivetrain<'a> {
    raw: NonNull<RawDrivetrain>,
    _deps: PhantomData<(&'a Motor, &'a Motor, &'a Inertial)>,
}

impl<'a> Drivetrain<'a> {
    pub fn new(
        l: &'a Motor,
        r: &'a Motor,
        wheel_travel: Distance,
        track_width: Distance,
        wheel_base: Distance,
        external_gear_ratio: f64,
    ) -> Option<Self> {
        unsafe {
            Some(Self {
                raw: NonNull::new(exp_drivetrain_new(
                    l.0.as_ptr(),
                    r.0.as_ptr(),
                    wheel_travel.mm(),
                    track_width.mm(),
                    wheel_base.mm(),
                    external_gear_ratio,
                ))?,
                _deps: PhantomData,
            })
        }
    }

    pub fn drive(&self, dir: Direction) {
        unsafe {
            exp_drivetrain_drive(self.raw.as_ptr(), dir);
        }
    }

    pub fn drive_velocity(&self, dir: Direction, velocity: Velocity) {
        unsafe {
            exp_drivetrain_drive_velocity(
                self.raw.as_ptr(),
                dir,
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn drive_for(&self, dir: Direction, distance: Distance, wait: bool) -> bool {
        unsafe { exp_drivetrain_drive_for(self.raw.as_ptr(), dir, distance.mm(), wait) }
    }

    pub fn drive_for_velocity(
        self,
        dir: Direction,
        distance: Distance,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_drivetrain_drive_for_velocity(
                self.raw.as_ptr(),
                dir,
                distance.mm(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn turn(&self, dir: TurnDirection) {
        unsafe {
            exp_drivetrain_turn(self.raw.as_ptr(), dir);
        }
    }

    pub fn turn_velocity(&self, dir: TurnDirection, velocity: Velocity) {
        unsafe {
            exp_drivetrain_turn_velocity(
                self.raw.as_ptr(),
                dir,
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn turn_for(&self, dir: TurnDirection, angle: Angle, wait: bool) -> bool {
        unsafe { exp_drivetrain_turn_for(self.raw.as_ptr(), dir, angle.deg(), wait) }
    }

    pub fn turn_for_velocity(
        &self,
        dir: TurnDirection,
        angle: Angle,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_drivetrain_turn_for_velocity(
                self.raw.as_ptr(),
                dir,
                angle.deg(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn stop(&self) {
        unsafe {
            exp_drivetrain_stop(self.raw.as_ptr());
        }
    }

    pub fn set_drive_velocity(&self, velocity: Velocity) {
        unsafe {
            exp_drivetrain_set_drive_velocity(
                self.raw.as_ptr(),
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn set_turn_velocity(&self, velocity: Velocity) {
        unsafe {
            exp_drivetrain_set_turn_velocity(self.raw.as_ptr(), velocity.value(), velocity.units());
        }
    }
}

impl Drop for Drivetrain<'_> {
    fn drop(&mut self) {
        unsafe {
            exp_drivetrain_free(self.raw.as_ptr());
        }
    }
}

pub struct Smartdrive<'a> {
    raw: NonNull<RawSmartdrive>,
    _deps: PhantomData<(&'a Motor, &'a Motor, &'a Inertial)>,
}

impl<'a> Smartdrive<'a> {
    pub fn new(
        l: &'a Motor,
        r: &'a Motor,
        i: &'a Inertial,
        wheel_travel: Distance,
        track_width: Distance,
        wheel_base: Distance,
        external_gear_ratio: f64,
    ) -> Option<Self> {
        unsafe {
            Some(Self {
                raw: NonNull::new(exp_smartdrive_new(
                    l.0.as_ptr(),
                    r.0.as_ptr(),
                    i.0.as_ptr(),
                    wheel_travel.mm(),
                    track_width.mm(),
                    wheel_base.mm(),
                    external_gear_ratio,
                ))?,
                _deps: PhantomData,
            })
        }
    }

    pub fn drive(&self, dir: Direction) {
        unsafe {
            exp_smartdrive_drive(self.raw.as_ptr(), dir);
        }
    }

    pub fn drive_velocity(&self, dir: Direction, velocity: Velocity) {
        unsafe {
            exp_smartdrive_drive_velocity(
                self.raw.as_ptr(),
                dir,
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn drive_for(&self, dir: Direction, distance: Distance, wait: bool) -> bool {
        unsafe { exp_smartdrive_drive_for(self.raw.as_ptr(), dir, distance.mm(), wait) }
    }

    pub fn drive_for_velocity(
        self,
        dir: Direction,
        distance: Distance,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_smartdrive_drive_for_velocity(
                self.raw.as_ptr(),
                dir,
                distance.mm(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn turn(&self, dir: TurnDirection) {
        unsafe {
            exp_smartdrive_turn(self.raw.as_ptr(), dir);
        }
    }

    pub fn turn_velocity(&self, dir: TurnDirection, velocity: Velocity) {
        unsafe {
            exp_smartdrive_turn_velocity(
                self.raw.as_ptr(),
                dir,
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn turn_for(&self, dir: TurnDirection, angle: Angle, wait: bool) -> bool {
        unsafe { exp_smartdrive_turn_for(self.raw.as_ptr(), dir, angle.deg(), wait) }
    }

    pub fn turn_for_velocity(
        &self,
        dir: TurnDirection,
        angle: Angle,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_smartdrive_turn_for_velocity(
                self.raw.as_ptr(),
                dir,
                angle.deg(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn stop(&self) {
        unsafe {
            exp_smartdrive_stop(self.raw.as_ptr());
        }
    }

    pub fn set_drive_velocity(&self, velocity: Velocity) {
        unsafe {
            exp_smartdrive_set_drive_velocity(
                self.raw.as_ptr(),
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn set_turn_velocity(&self, velocity: Velocity) {
        unsafe {
            exp_smartdrive_set_turn_velocity(self.raw.as_ptr(), velocity.value(), velocity.units());
        }
    }

    pub fn turn_to_heading(&self, angle: Angle) {
        unsafe {
            exp_smartdrive_turn_to_heading(self.raw.as_ptr(), angle.deg());
        }
    }

    pub fn turn_to_rotation(&self, angle: Angle) {
        unsafe {
            exp_smartdrive_turn_to_rotation(self.raw.as_ptr(), angle.deg());
        }
    }

    pub fn set_heading(&self, value: Angle) {
        unsafe {
            exp_smartdrive_set_heading(self.raw.as_ptr(), value.deg());
        }
    }

    pub fn set_rotation(&self, value: Angle) {
        unsafe {
            exp_smartdrive_set_rotation(self.raw.as_ptr(), value.deg());
        }
    }

    pub fn heading(&self) -> Angle {
        unsafe { exp_smartdrive_heading(self.raw.as_ptr()).deg() }
    }

    pub fn rotation(&self) -> Angle {
        unsafe { exp_smartdrive_rotation(self.raw.as_ptr()).deg() }
    }
}

impl Drop for Smartdrive<'_> {
    fn drop(&mut self) {
        unsafe {
            exp_smartdrive_free(self.raw.as_ptr());
        }
    }
}

pub struct Motor(NonNull<RawMotor>);

impl Motor {
    pub fn new(port: u8, reverse: bool) -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_motor_new(port, reverse))?)) }
    }

    pub fn spin(&self, dir: Direction) {
        unsafe {
            exp_motor_spin(self.0.as_ptr(), dir);
        }
    }

    pub fn spin_velocity(&self, dir: Direction, velocity: Velocity) {
        unsafe {
            exp_motor_spin_velocity(self.0.as_ptr(), dir, velocity.value(), velocity.units());
        }
    }

    pub fn spin_volts(&self, dir: Direction, voltage: Voltage) {
        unsafe {
            exp_motor_spin_volts(self.0.as_ptr(), dir, voltage.volts());
        }
    }

    pub fn spin_for(&self, dir: Direction, rotation: Angle, wait: bool) -> bool {
        unsafe { exp_motor_spin_for(self.0.as_ptr(), dir, rotation.deg(), wait) }
    }

    pub fn spin_for_velocity(
        &self,
        dir: Direction,
        rotation: Angle,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_motor_spin_for_velocity(
                self.0.as_ptr(),
                dir,
                rotation.deg(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn spin_for_time(&self, dir: Direction, time: Time) -> bool {
        unsafe { exp_motor_spin_for_time(self.0.as_ptr(), dir, time.ms()) }
    }

    pub fn spin_for_time_velocity(
        &self,
        dir: Direction,
        time: Time,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_motor_spin_for_time_velocity(
                self.0.as_ptr(),
                dir,
                time.ms(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn spin_to_position(&self, rotation: Angle, wait: bool) -> bool {
        unsafe { exp_motor_spin_to_position(self.0.as_ptr(), rotation.deg(), wait) }
    }

    pub fn spin_to_position_velocity(
        &self,
        rotation: Angle,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_motor_spin_to_position_velocity(
                self.0.as_ptr(),
                rotation.deg(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn stop(&self) {
        unsafe {
            exp_motor_stop(self.0.as_ptr());
        }
    }

    pub fn stop_mode(&self, mode: BrakeType) {
        unsafe {
            exp_motor_stop_mode(self.0.as_ptr(), mode);
        }
    }

    pub fn set_position(&self, value: Angle) {
        unsafe {
            exp_motor_set_position(self.0.as_ptr(), value.deg());
        }
    }

    pub fn set_velocity(&self, velocity: Velocity) {
        unsafe {
            exp_motor_set_velocity(self.0.as_ptr(), velocity.value(), velocity.units());
        }
    }

    pub fn set_stopping(&self, mode: BrakeType) {
        unsafe {
            exp_motor_set_stopping(self.0.as_ptr(), mode);
        }
    }

    pub fn set_max_torque(&self, value: Torque) {
        unsafe {
            exp_motor_set_max_torque(self.0.as_ptr(), value.newton_meters());
        }
    }

    pub fn set_max_torque_current(&self, value: Current) {
        unsafe {
            exp_motor_set_max_torque_current(self.0.as_ptr(), value.amps());
        }
    }

    pub fn set_max_torque_percent(&self, value: f64) {
        unsafe {
            exp_motor_set_max_torque_percent(self.0.as_ptr(), value);
        }
    }

    pub fn set_timeout(&self, time: Time) {
        unsafe {
            exp_motor_set_timeout(self.0.as_ptr(), round(time.ms()) as i32);
        }
    }

    pub fn is_done(&self) -> bool {
        unsafe { exp_motor_is_done(self.0.as_ptr()) }
    }

    pub fn is_spinning(&self) -> bool {
        unsafe { exp_motor_is_spinning(self.0.as_ptr()) }
    }

    pub fn position(&self) -> Angle {
        unsafe { exp_motor_position(self.0.as_ptr()).deg() }
    }

    pub fn velocity(&self, units: VelocityUnits) -> Velocity {
        unsafe { Velocity::from_value(exp_motor_velocity(self.0.as_ptr(), units), units) }
    }

    pub fn current(&self) -> Current {
        unsafe { exp_motor_current(self.0.as_ptr()).amps() }
    }

    pub fn current_percent(&self) -> f64 {
        unsafe { exp_motor_current_percent(self.0.as_ptr()) }
    }

    pub fn power(&self) -> Power {
        unsafe { exp_motor_power(self.0.as_ptr()).watts() }
    }

    pub fn torque(&self) -> Torque {
        unsafe { exp_motor_torque(self.0.as_ptr()).newton_meters() }
    }

    pub fn efficiency(&self) -> f64 {
        unsafe { exp_motor_efficiency(self.0.as_ptr()) }
    }

    pub fn temperature(&self) -> Temperature {
        unsafe { exp_motor_temperature(self.0.as_ptr()).celsius() }
    }

    pub fn temperature_percent(&self) -> f64 {
        unsafe { exp_motor_temperature_percent(self.0.as_ptr()) }
    }

    pub fn voltage(&self) -> Voltage {
        unsafe { exp_motor_voltage(self.0.as_ptr()).volts() }
    }

    pub fn direction(&self) -> Direction {
        unsafe { exp_motor_direction(self.0.as_ptr()) }
    }

    pub fn installed(&self) -> bool {
        unsafe { exp_motor_installed(self.0.as_ptr()) }
    }
}

impl Drop for Motor {
    fn drop(&mut self) {
        unsafe {
            exp_motor_free(self.0.as_ptr());
        }
    }
}

pub struct MotorGroup<'a> {
    raw: NonNull<RawMotorGroup>,
    _deps: PhantomData<&'a Motor>,
}

impl<'a> MotorGroup<'a> {
    pub fn new(motors: &[&'a Motor]) -> Option<Self> {
        let raw: Vec<_> = motors.iter().map(|m| m.0.as_ptr()).collect();
        unsafe {
            Some(Self {
                raw: NonNull::new(exp_motor_group_new(raw.as_ptr(), raw.len()))?,
                _deps: PhantomData,
            })
        }
    }

    pub fn spin(&self, dir: Direction) {
        unsafe {
            exp_motor_group_spin(self.raw.as_ptr(), dir);
        }
    }

    pub fn spin_velocity(&self, dir: Direction, velocity: Velocity) {
        unsafe {
            exp_motor_group_spin_velocity(
                self.raw.as_ptr(),
                dir,
                velocity.value(),
                velocity.units(),
            );
        }
    }

    pub fn spin_volts(&self, dir: Direction, voltage: Voltage) {
        unsafe {
            exp_motor_group_spin_volts(self.raw.as_ptr(), dir, voltage.volts());
        }
    }

    pub fn spin_for(&self, dir: Direction, rotation: Angle, wait: bool) -> bool {
        unsafe { exp_motor_group_spin_for(self.raw.as_ptr(), dir, rotation.deg(), wait) }
    }

    pub fn spin_for_velocity(
        &self,
        dir: Direction,
        rotation: Angle,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_motor_group_spin_for_velocity(
                self.raw.as_ptr(),
                dir,
                rotation.deg(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn spin_for_time(&self, dir: Direction, time: Time) {
        unsafe {
            exp_motor_group_spin_for_time(self.raw.as_ptr(), dir, time.ms());
        }
    }

    pub fn spin_for_time_velocity(
        &self,
        dir: Direction,
        time: Time,
        velocity: Velocity,
        wait: bool,
    ) {
        unsafe {
            exp_motor_group_spin_for_time_velocity(
                self.raw.as_ptr(),
                dir,
                time.ms(),
                velocity.value(),
                velocity.units(),
                wait,
            );
        }
    }

    pub fn spin_to_position(&self, rotation: Angle, wait: bool) -> bool {
        unsafe { exp_motor_group_spin_to_position(self.raw.as_ptr(), rotation.deg(), wait) }
    }

    pub fn spin_to_position_velocity(
        &self,
        rotation: Angle,
        velocity: Velocity,
        wait: bool,
    ) -> bool {
        unsafe {
            exp_motor_group_spin_to_position_velocity(
                self.raw.as_ptr(),
                rotation.deg(),
                velocity.value(),
                velocity.units(),
                wait,
            )
        }
    }

    pub fn stop(&self) {
        unsafe {
            exp_motor_group_stop(self.raw.as_ptr());
        }
    }

    pub fn stop_mode(&self, mode: BrakeType) {
        unsafe {
            exp_motor_group_stop_mode(self.raw.as_ptr(), mode);
        }
    }

    pub fn set_position(&self, value: Angle) {
        unsafe {
            exp_motor_group_set_position(self.raw.as_ptr(), value.deg());
        }
    }

    pub fn set_velocity(&self, velocity: Velocity) {
        unsafe {
            exp_motor_group_set_velocity(self.raw.as_ptr(), velocity.value(), velocity.units());
        }
    }

    pub fn set_stopping(&self, mode: BrakeType) {
        unsafe {
            exp_motor_group_set_stopping(self.raw.as_ptr(), mode);
        }
    }

    pub fn set_max_torque(&self, value: Torque) {
        unsafe {
            exp_motor_group_set_max_torque(self.raw.as_ptr(), value.newton_meters());
        }
    }

    pub fn set_max_torque_current(&self, value: Current) {
        unsafe {
            exp_motor_group_set_max_torque_current(self.raw.as_ptr(), value.amps());
        }
    }

    pub fn set_max_torque_percent(&self, value: f64) {
        unsafe {
            exp_motor_group_set_max_torque_percent(self.raw.as_ptr(), value);
        }
    }

    pub fn set_timeout(&self, time: Time) {
        unsafe {
            exp_motor_group_set_timeout(self.raw.as_ptr(), round(time.ms()) as i32);
        }
    }

    pub fn is_done(&self) -> bool {
        unsafe { exp_motor_group_is_done(self.raw.as_ptr()) }
    }

    pub fn is_spinning(&self) -> bool {
        unsafe { exp_motor_group_is_spinning(self.raw.as_ptr()) }
    }

    pub fn position(&self) -> Angle {
        unsafe { exp_motor_group_position(self.raw.as_ptr()).deg() }
    }

    pub fn velocity(&self, units: VelocityUnits) -> Velocity {
        unsafe { Velocity::from_value(exp_motor_group_velocity(self.raw.as_ptr(), units), units) }
    }

    pub fn current(&self) -> Current {
        unsafe { exp_motor_group_current(self.raw.as_ptr()).amps() }
    }

    pub fn current_percent(&self) -> f64 {
        unsafe { exp_motor_group_current_percent(self.raw.as_ptr()) }
    }

    pub fn power(&self) -> Power {
        unsafe { exp_motor_group_power(self.raw.as_ptr()).watts() }
    }

    pub fn torque(&self) -> Torque {
        unsafe { exp_motor_group_torque(self.raw.as_ptr()).newton_meters() }
    }

    pub fn efficiency(&self) -> f64 {
        unsafe { exp_motor_group_efficiency(self.raw.as_ptr()) }
    }

    pub fn temperature(&self) -> Temperature {
        unsafe { exp_motor_group_temperature(self.raw.as_ptr()).celsius() }
    }

    pub fn temperature_percent(&self) -> f64 {
        unsafe { exp_motor_group_temperature_percent(self.raw.as_ptr()) }
    }

    pub fn voltage(&self) -> Voltage {
        unsafe { exp_motor_group_voltage(self.raw.as_ptr()).volts() }
    }

    pub fn direction(&self) -> Direction {
        unsafe { exp_motor_group_direction(self.raw.as_ptr()) }
    }

    pub fn count(&self) -> i32 {
        unsafe { exp_motor_group_count(self.raw.as_ptr()) }
    }
}

impl Drop for MotorGroup<'_> {
    fn drop(&mut self) {
        unsafe {
            exp_motor_group_free(self.raw.as_ptr());
        }
    }
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

    pub fn angle(&self) -> Angle {
        unsafe { exp_inertial_angle(self.0.as_ptr()).deg() }
    }
}

impl Drop for Inertial {
    fn drop(&mut self) {
        unsafe {
            exp_inertial_free(self.0.as_ptr());
        }
    }
}

pub struct DistanceSensor(NonNull<RawDistance>);

impl DistanceSensor {
    pub fn new(port: u8) -> Option<Self> {
        unsafe { Some(Self(NonNull::new(exp_distance_new(port))?)) }
    }

    pub fn is_object_detected(&self) -> bool {
        unsafe { exp_distance_is_object_detected(self.0.as_ptr()) }
    }

    pub fn object_distance(&self) -> Distance {
        unsafe { exp_distance_object_distance(self.0.as_ptr()).mm() }
    }
}

impl Drop for DistanceSensor {
    fn drop(&mut self) {
        unsafe {
            exp_distance_free(self.0.as_ptr());
        }
    }
}

pub struct SmartdriveBuilder {
    wheel_travel: Distance,
    track_width: Distance,
    wheel_base: Distance,
    external_gear_ratio: f64,
}

impl SmartdriveBuilder {
    pub fn new() -> Self {
        Self {
            wheel_travel: 320.mm(),
            track_width: 320.mm(),
            wheel_base: 130.mm(),
            external_gear_ratio: 1.0,
        }
    }

    pub fn wheel_travel(&mut self, wheel_travel: Distance) -> &mut Self {
        self.wheel_travel = wheel_travel;
        self
    }

    pub fn track_width(&mut self, track_width: Distance) -> &mut Self {
        self.track_width = track_width;
        self
    }

    pub fn wheel_base(&mut self, wheel_base: Distance) -> &mut Self {
        self.wheel_base = wheel_base;
        self
    }

    pub fn external_gear_ratio(&mut self, external_gear_ratio: f64) -> &mut Self {
        self.external_gear_ratio = external_gear_ratio;
        self
    }

    pub fn init<'a>(&self, l: &'a Motor, r: &'a Motor, i: &'a Inertial) -> Option<Smartdrive<'a>> {
        Smartdrive::new(
            l,
            r,
            i,
            self.wheel_travel,
            self.track_width,
            self.wheel_base,
            self.external_gear_ratio,
        )
    }
}

impl Default for SmartdriveBuilder {
    fn default() -> Self {
        Self::new()
    }
}
