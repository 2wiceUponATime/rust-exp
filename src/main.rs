#![no_std]
#![no_main]

use rust_exp::{
    Brain, Direction, DistanceSensor, Inertial, Motor, SmartdriveBuilder, TurnDirection,
    screen_print_at, sleep, units::Units,
};

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let brain = Brain::new().unwrap();
    let inertial = Inertial::new().unwrap();
    screen_print_at!(brain, 0, 30, "Calibrating").unwrap();
    inertial.calibrate();
    while inertial.is_calibrating() {
        sleep(25);
    }
    brain.screen_clear();
    let left_drive = Motor::new(10, false).unwrap();
    let right_drive = Motor::new(6, true).unwrap();
    let distance = DistanceSensor::new(1).unwrap();
    let drive = SmartdriveBuilder::new()
        .init(&left_drive, &right_drive, &inertial)
        .unwrap();
    drive.set_drive_velocity(100.percent());
    drive.set_turn_velocity(20.percent());
    loop {
        drive.turn(TurnDirection::Right);
        while !(distance.is_object_detected() && distance.object_distance() < 20.inches()) {
            sleep(10);
        }
        drive.drive_for(Direction::Forward, 20.inches(), true);
        drive.drive_for(Direction::Reverse, 20.inches(), true);
    }
}
