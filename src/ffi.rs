use super::*;
use core::{
    ffi::c_char,
    marker::{PhantomData, PhantomPinned},
};

macro_rules! opaque {
    ($($name:ident),+ $(,)?) => {
        $(
            #[repr(C)]
            pub struct $name {
                _data: [u8; 0],
                _marker: PhantomData<(*mut u8, PhantomPinned)>,
            }
        )+
    };
}

opaque!(
    RawDrivetrain,
    RawSmartdrive,
    RawMotor,
    RawBrain,
    RawInertial,
    RawDistance
);

unsafe extern "C" {
    pub fn exp_time() -> u32;
    pub fn exp_printf(s: *const c_char) -> ();
    pub fn exp_flush_stdout() -> i32;
    pub fn exp_thread_sleep(time: u32) -> ();
    pub fn exp_request_exit() -> ();
    pub fn exp_drivetrain_new(
        l: *mut RawMotor,
        r: *mut RawMotor,
        wheel_travel: f64,
        track_width: f64,
        wheel_base: f64,
        external_gear_ratio: f64,
    ) -> *mut RawDrivetrain;
    pub fn exp_drivetrain_free(d: *mut RawDrivetrain) -> ();
    pub fn exp_drivetrain_drive(d: *mut RawDrivetrain, dir: DriveDirection) -> ();
    pub fn exp_drivetrain_drive_velocity(
        d: *mut RawDrivetrain,
        dir: DriveDirection,
        velocity: f64,
    ) -> ();
    pub fn exp_drivetrain_drive_for(
        d: *mut RawDrivetrain,
        dir: DriveDirection,
        distance: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_drive_for_velocity(
        d: *mut RawDrivetrain,
        dir: DriveDirection,
        distance: f64,
        velocity: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_turn(d: *mut RawDrivetrain, dir: TurnDirection) -> ();
    pub fn exp_drivetrain_turn_velocity(
        d: *mut RawDrivetrain,
        dir: TurnDirection,
        velocity: f64,
    ) -> ();
    pub fn exp_drivetrain_turn_for(
        d: *mut RawDrivetrain,
        dir: TurnDirection,
        angle: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_turn_for_velocity(
        d: *mut RawDrivetrain,
        dir: TurnDirection,
        angle: f64,
        velocity: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_stop(d: *mut RawDrivetrain) -> ();
    pub fn exp_drivetrain_set_drive_velocity(d: *mut RawDrivetrain, velocity: f64) -> ();
    pub fn exp_drivetrain_set_turn_velocity(d: *mut RawDrivetrain, velocity: f64) -> ();
    pub fn exp_smartdrive_new(
        l: *mut RawMotor,
        r: *mut RawMotor,
        i: *mut RawInertial,
        wheel_travel: f64,
        track_width: f64,
        wheel_base: f64,
        external_gear_ratio: f64,
    ) -> *mut RawSmartdrive;
    pub fn exp_smartdrive_free(s: *mut RawSmartdrive) -> ();
    pub fn exp_smartdrive_drive(d: *mut RawSmartdrive, dir: DriveDirection) -> ();
    pub fn exp_smartdrive_drive_velocity(
        d: *mut RawSmartdrive,
        dir: DriveDirection,
        velocity: f64,
    ) -> ();
    pub fn exp_smartdrive_drive_for(
        d: *mut RawSmartdrive,
        dir: DriveDirection,
        distance: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_drive_for_velocity(
        d: *mut RawSmartdrive,
        dir: DriveDirection,
        distance: f64,
        velocity: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_turn(d: *mut RawSmartdrive, dir: TurnDirection) -> ();
    pub fn exp_smartdrive_turn_velocity(
        d: *mut RawSmartdrive,
        dir: TurnDirection,
        velocity: f64,
    ) -> ();
    pub fn exp_smartdrive_turn_for(
        d: *mut RawSmartdrive,
        dir: TurnDirection,
        angle: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_turn_for_velocity(
        d: *mut RawSmartdrive,
        dir: TurnDirection,
        angle: f64,
        velocity: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_stop(d: *mut RawSmartdrive) -> ();
    pub fn exp_smartdrive_set_drive_velocity(d: *mut RawSmartdrive, velocity: f64) -> ();
    pub fn exp_smartdrive_set_turn_velocity(d: *mut RawSmartdrive, velocity: f64) -> ();
    pub fn exp_smartdrive_turn_to_heading(s: *mut RawSmartdrive, angle: f64) -> ();
    pub fn exp_smartdrive_turn_to_rotation(s: *mut RawSmartdrive, angle: f64) -> ();
    pub fn exp_smartdrive_set_heading(s: *mut RawSmartdrive, value: f64) -> ();
    pub fn exp_smartdrive_set_rotation(s: *mut RawSmartdrive, value: f64) -> ();
    pub fn exp_smartdrive_heading(s: *mut RawSmartdrive) -> f64;
    pub fn exp_smartdrive_rotation(s: *mut RawSmartdrive) -> f64;
    pub fn exp_motor_new(port: u8, reverse: bool) -> *mut RawMotor;
    pub fn exp_motor_free(m: *mut RawMotor) -> ();
    pub fn exp_motor_spin(m: *mut RawMotor, p: f64) -> ();
    pub fn exp_brain_new() -> *mut RawBrain;
    pub fn exp_brain_free(b: *mut RawBrain) -> ();
    pub fn exp_brain_screen_clear(brain: *mut RawBrain) -> ();
    pub fn exp_brain_screen_print_at(brain: *mut RawBrain, x: i32, y: i32, s: *const c_char) -> ();
    pub fn exp_inertial_new() -> *mut RawInertial;
    pub fn exp_inertial_new_port(port: u8) -> *mut RawInertial;
    pub fn exp_inertial_free(i: *mut RawInertial) -> ();
    pub fn exp_inertial_calibrate(i: *mut RawInertial) -> ();
    pub fn exp_inertial_is_calibrating(i: *mut RawInertial) -> bool;
    pub fn exp_inertial_angle(i: *mut RawInertial) -> f64;
    pub fn exp_distance_new(port: u8) -> *mut RawDistance;
    pub fn exp_distance_free(d: *mut RawDistance) -> ();
    pub fn exp_distance_is_object_detected(d: *mut RawDistance) -> bool;
    pub fn exp_distance_object_distance(d: *mut RawDistance) -> f64;
}
