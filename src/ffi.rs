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
    RawMotorGroup,
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
    pub fn exp_drivetrain_drive(d: *mut RawDrivetrain, dir: Direction) -> ();
    pub fn exp_drivetrain_drive_velocity(
        d: *mut RawDrivetrain,
        dir: Direction,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_drivetrain_drive_for(
        d: *mut RawDrivetrain,
        dir: Direction,
        distance: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_drive_for_velocity(
        d: *mut RawDrivetrain,
        dir: Direction,
        distance: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_turn(d: *mut RawDrivetrain, dir: TurnDirection) -> ();
    pub fn exp_drivetrain_turn_velocity(
        d: *mut RawDrivetrain,
        dir: TurnDirection,
        velocity: f64,
        units: VelocityUnits,
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
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_drivetrain_stop(d: *mut RawDrivetrain) -> ();
    pub fn exp_drivetrain_set_drive_velocity(
        d: *mut RawDrivetrain,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_drivetrain_set_turn_velocity(
        d: *mut RawDrivetrain,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
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
    pub fn exp_smartdrive_drive(d: *mut RawSmartdrive, dir: Direction) -> ();
    pub fn exp_smartdrive_drive_velocity(
        d: *mut RawSmartdrive,
        dir: Direction,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_smartdrive_drive_for(
        d: *mut RawSmartdrive,
        dir: Direction,
        distance: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_drive_for_velocity(
        d: *mut RawSmartdrive,
        dir: Direction,
        distance: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_turn(d: *mut RawSmartdrive, dir: TurnDirection) -> ();
    pub fn exp_smartdrive_turn_velocity(
        d: *mut RawSmartdrive,
        dir: TurnDirection,
        velocity: f64,
        units: VelocityUnits,
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
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_smartdrive_stop(d: *mut RawSmartdrive) -> ();
    pub fn exp_smartdrive_set_drive_velocity(
        d: *mut RawSmartdrive,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_smartdrive_set_turn_velocity(
        d: *mut RawSmartdrive,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_smartdrive_turn_to_heading(s: *mut RawSmartdrive, angle: f64) -> ();
    pub fn exp_smartdrive_turn_to_rotation(s: *mut RawSmartdrive, angle: f64) -> ();
    pub fn exp_smartdrive_set_heading(s: *mut RawSmartdrive, value: f64) -> ();
    pub fn exp_smartdrive_set_rotation(s: *mut RawSmartdrive, value: f64) -> ();
    pub fn exp_smartdrive_heading(s: *mut RawSmartdrive) -> f64;
    pub fn exp_smartdrive_rotation(s: *mut RawSmartdrive) -> f64;
    pub fn exp_motor_new(port: u8, reverse: bool) -> *mut RawMotor;
    pub fn exp_motor_free(m: *mut RawMotor) -> ();
    pub fn exp_motor_spin(m: *mut RawMotor, dir: Direction) -> ();
    pub fn exp_motor_spin_velocity(
        m: *mut RawMotor,
        dir: Direction,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_motor_spin_volts(m: *mut RawMotor, dir: Direction, voltage: f64) -> ();
    pub fn exp_motor_spin_for(m: *mut RawMotor, dir: Direction, rotation: f64, wait: bool) -> bool;
    pub fn exp_motor_spin_for_velocity(
        m: *mut RawMotor,
        dir: Direction,
        rotation: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_spin_for_time(m: *mut RawMotor, dir: Direction, time: f64) -> bool;
    pub fn exp_motor_spin_for_time_velocity(
        m: *mut RawMotor,
        dir: Direction,
        time: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_spin_to_position(m: *mut RawMotor, rotation: f64, wait: bool) -> bool;
    pub fn exp_motor_spin_to_position_velocity(
        m: *mut RawMotor,
        rotation: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_stop(m: *mut RawMotor) -> ();
    pub fn exp_motor_stop_mode(m: *mut RawMotor, mode: BrakeType) -> ();
    pub fn exp_motor_set_position(m: *mut RawMotor, value: f64) -> ();
    pub fn exp_motor_set_velocity(m: *mut RawMotor, velocity: f64, units: VelocityUnits) -> ();
    pub fn exp_motor_set_stopping(m: *mut RawMotor, mode: BrakeType) -> ();
    pub fn exp_motor_set_max_torque(m: *mut RawMotor, value: f64) -> ();
    pub fn exp_motor_set_max_torque_current(m: *mut RawMotor, value: f64) -> ();
    pub fn exp_motor_set_max_torque_percent(m: *mut RawMotor, value: f64) -> ();
    pub fn exp_motor_set_timeout(m: *mut RawMotor, time: i32) -> ();
    pub fn exp_motor_is_done(m: *mut RawMotor) -> bool;
    pub fn exp_motor_is_spinning(m: *mut RawMotor) -> bool;
    pub fn exp_motor_position(m: *mut RawMotor) -> f64;
    pub fn exp_motor_velocity(m: *mut RawMotor, units: VelocityUnits) -> f64;
    pub fn exp_motor_current(m: *mut RawMotor) -> f64;
    pub fn exp_motor_current_percent(m: *mut RawMotor) -> f64;
    pub fn exp_motor_power(m: *mut RawMotor) -> f64;
    pub fn exp_motor_torque(m: *mut RawMotor) -> f64;
    pub fn exp_motor_efficiency(m: *mut RawMotor) -> f64;
    pub fn exp_motor_temperature(m: *mut RawMotor) -> f64;
    pub fn exp_motor_temperature_percent(m: *mut RawMotor) -> f64;
    pub fn exp_motor_voltage(m: *mut RawMotor) -> f64;
    pub fn exp_motor_direction(m: *mut RawMotor) -> Direction;
    pub fn exp_motor_installed(m: *mut RawMotor) -> bool;
    pub fn exp_motor_group_new(motors: *const *mut RawMotor, len: usize) -> *mut RawMotorGroup;
    pub fn exp_motor_group_free(g: *mut RawMotorGroup) -> ();
    pub fn exp_motor_group_spin(m: *mut RawMotorGroup, dir: Direction) -> ();
    pub fn exp_motor_group_spin_velocity(
        m: *mut RawMotorGroup,
        dir: Direction,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_motor_group_spin_volts(m: *mut RawMotorGroup, dir: Direction, voltage: f64) -> ();
    pub fn exp_motor_group_spin_for(
        m: *mut RawMotorGroup,
        dir: Direction,
        rotation: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_group_spin_for_velocity(
        m: *mut RawMotorGroup,
        dir: Direction,
        rotation: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_group_spin_for_time(m: *mut RawMotorGroup, dir: Direction, time: f64) -> ();
    pub fn exp_motor_group_spin_for_time_velocity(
        m: *mut RawMotorGroup,
        dir: Direction,
        time: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> ();
    pub fn exp_motor_group_spin_to_position(
        m: *mut RawMotorGroup,
        rotation: f64,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_group_spin_to_position_velocity(
        m: *mut RawMotorGroup,
        rotation: f64,
        velocity: f64,
        units: VelocityUnits,
        wait: bool,
    ) -> bool;
    pub fn exp_motor_group_stop(m: *mut RawMotorGroup) -> ();
    pub fn exp_motor_group_stop_mode(m: *mut RawMotorGroup, mode: BrakeType) -> ();
    pub fn exp_motor_group_set_position(m: *mut RawMotorGroup, value: f64) -> ();
    pub fn exp_motor_group_set_velocity(
        m: *mut RawMotorGroup,
        velocity: f64,
        units: VelocityUnits,
    ) -> ();
    pub fn exp_motor_group_set_stopping(m: *mut RawMotorGroup, mode: BrakeType) -> ();
    pub fn exp_motor_group_set_max_torque(m: *mut RawMotorGroup, value: f64) -> ();
    pub fn exp_motor_group_set_max_torque_current(m: *mut RawMotorGroup, value: f64) -> ();
    pub fn exp_motor_group_set_max_torque_percent(m: *mut RawMotorGroup, value: f64) -> ();
    pub fn exp_motor_group_set_timeout(m: *mut RawMotorGroup, time: i32) -> ();
    pub fn exp_motor_group_is_done(m: *mut RawMotorGroup) -> bool;
    pub fn exp_motor_group_is_spinning(m: *mut RawMotorGroup) -> bool;
    pub fn exp_motor_group_position(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_velocity(m: *mut RawMotorGroup, units: VelocityUnits) -> f64;
    pub fn exp_motor_group_current(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_current_percent(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_power(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_torque(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_efficiency(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_temperature(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_temperature_percent(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_voltage(m: *mut RawMotorGroup) -> f64;
    pub fn exp_motor_group_direction(m: *mut RawMotorGroup) -> Direction;
    pub fn exp_motor_group_count(m: *mut RawMotorGroup) -> i32;
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
