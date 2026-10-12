#include <stdio.h>

#include "exp_cpp.h"

#define PORT_CASE(i, c, ...) \
  case i:                    \
    return new c(PORT##i __VA_OPT__(, ) __VA_ARGS__)
#define PORT_CASES(...)       \
  PORT_CASE(1, __VA_ARGS__);  \
  PORT_CASE(2, __VA_ARGS__);  \
  PORT_CASE(3, __VA_ARGS__);  \
  PORT_CASE(4, __VA_ARGS__);  \
  PORT_CASE(5, __VA_ARGS__);  \
  PORT_CASE(6, __VA_ARGS__);  \
  PORT_CASE(7, __VA_ARGS__);  \
  PORT_CASE(8, __VA_ARGS__);  \
  PORT_CASE(9, __VA_ARGS__);  \
  PORT_CASE(10, __VA_ARGS__); \
  PORT_CASE(11, __VA_ARGS__); \
  PORT_CASE(12, __VA_ARGS__); \
  PORT_CASE(13, __VA_ARGS__); \
  PORT_CASE(14, __VA_ARGS__); \
  PORT_CASE(15, __VA_ARGS__); \
  PORT_CASE(16, __VA_ARGS__); \
  PORT_CASE(17, __VA_ARGS__); \
  PORT_CASE(18, __VA_ARGS__); \
  PORT_CASE(19, __VA_ARGS__); \
  PORT_CASE(20, __VA_ARGS__); \
  PORT_CASE(21, __VA_ARGS__); \
  PORT_CASE(22, __VA_ARGS__)

using namespace vex;

enum class DriveDirection : uint8_t {
  Forward = 0,
  Reverse = 1,
};

vex::directionType from_dir(DriveDirection dir) {
  return dir == DriveDirection::Forward ? forward : reverse;
}

enum class TurnDirection : uint8_t {
  Left = 0,
  Right = 1,
};

vex::turnType from_dir(TurnDirection dir) {
  return dir == TurnDirection::Left ? left : right;
}

extern "C" {
// System
uint32_t exp_time() { return vex::timer::system(); }

void exp_printf(char* s) { printf("%s", s); }

int exp_flush_stdout() { return fflush(stdout); }

void exp_thread_sleep(uint32_t time) { this_thread::sleep_for(time); }

void exp_request_exit() { vexSystemExitRequest(); }

// Drivetrain
drivetrain* exp_drivetrain_new(motor* l, motor* r, double wheel_travel,
                               double track_width, double wheel_base,
                               double external_gear_ratio) {
  return new drivetrain(*l, *r, wheel_travel, track_width, wheel_base, mm,
                        external_gear_ratio);
}

void exp_drivetrain_free(drivetrain* d) { delete d; }

void exp_drivetrain_drive(drivetrain* d, DriveDirection dir) {
  d->drive(from_dir(dir));
}

void exp_drivetrain_drive_velocity(drivetrain* d, DriveDirection dir,
                                   double velocity) {
  d->drive(from_dir(dir), velocity, velocityUnits::pct);
}

bool exp_drivetrain_drive_for(drivetrain* d, DriveDirection dir,
                              double distance, bool wait) {
  return d->driveFor(from_dir(dir), distance, mm, wait);
}

bool exp_drivetrain_drive_for_velocity(drivetrain* d, DriveDirection dir,
                                       double distance, double velocity,
                                       bool wait) {
  return d->driveFor(from_dir(dir), distance, mm, velocity, velocityUnits::pct,
                     wait);
}

void exp_drivetrain_turn(drivetrain* d, TurnDirection dir) {
  d->turn(from_dir(dir));
}

void exp_drivetrain_turn_velocity(drivetrain* d, TurnDirection dir,
                                  double velocity) {
  d->turn(from_dir(dir), velocity, velocityUnits::pct);
}

bool exp_drivetrain_turn_for(drivetrain* d, TurnDirection dir, double angle,
                             bool wait) {
  return d->turnFor(from_dir(dir), angle, degrees, wait);
}

bool exp_drivetrain_turn_for_velocity(drivetrain* d, TurnDirection dir,
                                      double angle, double velocity,
                                      bool wait) {
  return d->turnFor(from_dir(dir), angle, degrees, velocity, velocityUnits::pct,
                    wait);
}

void exp_drivetrain_stop(drivetrain* d) { d->stop(); }

void exp_drivetrain_set_drive_velocity(drivetrain* d, double velocity) {
  d->setDriveVelocity(velocity, percent);
}

void exp_drivetrain_set_turn_velocity(drivetrain* d, double velocity) {
  d->setTurnVelocity(velocity, percent);
}

// Smartdrive
smartdrive* exp_smartdrive_new(motor* l, motor* r, inertial* i,
                               double wheel_travel, double track_width,
                               double wheel_base, double external_gear_ratio) {
  return new smartdrive(*l, *r, *i, wheel_travel, track_width, wheel_base, mm,
                        external_gear_ratio);
}

void exp_smartdrive_free(smartdrive* s) { delete s; }

void exp_smartdrive_drive(smartdrive* d, DriveDirection dir) {
  d->drive(from_dir(dir));
}

void exp_smartdrive_drive_velocity(smartdrive* d, DriveDirection dir,
                                   double velocity) {
  d->drive(from_dir(dir), velocity, velocityUnits::pct);
}

bool exp_smartdrive_drive_for(smartdrive* d, DriveDirection dir,
                              double distance, bool wait) {
  return d->driveFor(from_dir(dir), distance, mm, wait);
}

bool exp_smartdrive_drive_for_velocity(smartdrive* d, DriveDirection dir,
                                       double distance, double velocity,
                                       bool wait) {
  return d->driveFor(from_dir(dir), distance, mm, velocity, velocityUnits::pct,
                     wait);
}

void exp_smartdrive_turn(smartdrive* d, TurnDirection dir) {
  d->turn(from_dir(dir));
}

void exp_smartdrive_turn_velocity(smartdrive* d, TurnDirection dir,
                                  double velocity) {
  d->turn(from_dir(dir), velocity, velocityUnits::pct);
}

bool exp_smartdrive_turn_for(smartdrive* d, TurnDirection dir, double angle,
                             bool wait) {
  return d->turnFor(from_dir(dir), angle, degrees, wait);
}

bool exp_smartdrive_turn_for_velocity(smartdrive* d, TurnDirection dir,
                                      double angle, double velocity,
                                      bool wait) {
  return d->turnFor(from_dir(dir), angle, degrees, velocity, velocityUnits::pct,
                    wait);
}

void exp_smartdrive_stop(smartdrive* d) { d->stop(); }

void exp_smartdrive_set_drive_velocity(smartdrive* d, double velocity) {
  d->setDriveVelocity(velocity, percent);
}

void exp_smartdrive_set_turn_velocity(smartdrive* d, double velocity) {
  d->setTurnVelocity(velocity, percent);
}

void exp_smartdrive_turn_to_heading(smartdrive* s, double angle) {
  s->turnToHeading(angle, degrees);
}

void exp_smartdrive_turn_to_rotation(smartdrive* s, double angle) {
  s->turnToRotation(angle, degrees);
}

void exp_smartdrive_set_heading(smartdrive* s, double value) {
  s->setHeading(value, degrees);
}

void exp_smartdrive_set_rotation(smartdrive* s, double value) {
  s->setRotation(value, degrees);
}

double exp_smartdrive_heading(smartdrive* s) { return s->heading(); }

double exp_smartdrive_rotation(smartdrive* s) { return s->rotation(); }

// Motor
motor* exp_motor_new(uint8_t port, bool reverse) {
  switch (port) {
    PORT_CASES(motor, reverse);
    default:
      return 0;
  }
}

void exp_motor_free(motor* m) { delete m; }

void exp_motor_spin(motor* m, double p) { m->spin(forward, p, percent); }

// Brain
brain* exp_brain_new() { return new brain; }

void exp_brain_free(brain* b) { delete b; }

void exp_brain_screen_clear(brain* brain) { brain->Screen.clearScreen(); }

void exp_brain_screen_print_at(brain* brain, int32_t x, int32_t y, char* s) {
  brain->Screen.printAt(x, y, s);
}

// Inertial
inertial* exp_inertial_new() { return new inertial(); }

inertial* exp_inertial_new_port(uint8_t port) {
  switch (port) {
    PORT_CASES(inertial);
    default:
      return 0;
  }
}

void exp_inertial_free(inertial* i) { delete i; }

void exp_inertial_calibrate(inertial* i) { i->calibrate(); }

bool exp_inertial_is_calibrating(inertial* i) { return i->isCalibrating(); }

double exp_inertial_angle(inertial* i) { return i->angle(degrees); }

// Distance
distance* exp_distance_new(uint8_t port) {
  switch (port) {
    PORT_CASES(distance);
    default:
      return 0;
  }
}

void exp_distance_free(distance* d) { delete d; }

bool exp_distance_is_object_detected(distance* d) {
  return d->isObjectDetected();
}

double exp_distance_object_distance(distance* d) {
  return d->objectDistance(mm);
}
}
