#include <stdio.h>

#include <utility>

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

namespace {

enum class Direction : uint8_t {
  Forward = 0,
  Reverse = 1,
};

directionType from_dir(Direction dir) {
  return dir == Direction::Forward ? forward : reverse;
}

Direction to_dir(directionType dir) {
  return dir == forward ? Direction::Forward : Direction::Reverse;
}

enum class TurnDirection : uint8_t {
  Left = 0,
  Right = 1,
};

turnType from_dir(TurnDirection dir) {
  return dir == TurnDirection::Left ? left : right;
}

enum class BrakeType : uint8_t {
  Coast = 0,
  Brake = 1,
  Hold = 2,
};

brakeType from_brake(BrakeType b) {
  switch (b) {
    case BrakeType::Coast:
      return hold;
    case BrakeType::Brake:
      return brake;
    default:
      return hold;
  }
}

enum class VelocityUnits : uint8_t {
  Percent = 0,
  Rpm = 1,
};

velocityUnits from_vel(VelocityUnits vel) {
  return vel == VelocityUnits::Percent ? velocityUnits::pct : rpm;
}

}  // namespace

extern "C" {
// System
uint32_t exp_time() { return timer::system(); }

void exp_printf(char* s) { printf("%s", s); }

int exp_flush_stdout() { return fflush(stdout); }

void exp_thread_sleep(uint32_t time) { this_thread::sleep_for(time); }

void exp_request_exit() { vexSystemExitRequest(); }

// Drivetrain
drivetrain* exp_drivetrain_new(
  motor* l,
  motor* r,
  double wheel_travel,
  double track_width,
  double wheel_base,
  double external_gear_ratio
) {
  return new drivetrain(
    *l,
    *r,
    wheel_travel,
    track_width,
    wheel_base,
    mm,
    external_gear_ratio
  );
}

void exp_drivetrain_free(drivetrain* d) { delete d; }

void exp_drivetrain_drive(drivetrain* d, Direction dir) {
  d->drive(from_dir(dir));
}

void exp_drivetrain_drive_velocity(
  drivetrain* d,
  Direction dir,
  double velocity,
  VelocityUnits units
) {
  d->drive(from_dir(dir), velocity, from_vel(units));
}

bool exp_drivetrain_drive_for(
  drivetrain* d,
  Direction dir,
  double distance,
  bool wait
) {
  return d->driveFor(from_dir(dir), distance, mm, wait);
}

bool exp_drivetrain_drive_for_velocity(
  drivetrain* d,
  Direction dir,
  double distance,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return d
    ->driveFor(from_dir(dir), distance, mm, velocity, from_vel(units), wait);
}

void exp_drivetrain_turn(drivetrain* d, TurnDirection dir) {
  d->turn(from_dir(dir));
}

void exp_drivetrain_turn_velocity(
  drivetrain* d,
  TurnDirection dir,
  double velocity,
  VelocityUnits units
) {
  d->turn(from_dir(dir), velocity, from_vel(units));
}

bool exp_drivetrain_turn_for(
  drivetrain* d,
  TurnDirection dir,
  double angle,
  bool wait
) {
  return d->turnFor(from_dir(dir), angle, degrees, wait);
}

bool exp_drivetrain_turn_for_velocity(
  drivetrain* d,
  TurnDirection dir,
  double angle,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return d
    ->turnFor(from_dir(dir), angle, degrees, velocity, from_vel(units), wait);
}

void exp_drivetrain_stop(drivetrain* d) { d->stop(); }

void exp_drivetrain_set_drive_velocity(
  drivetrain* d,
  double velocity,
  VelocityUnits units
) {
  d->setDriveVelocity(velocity, from_vel(units));
}

void exp_drivetrain_set_turn_velocity(
  drivetrain* d,
  double velocity,
  VelocityUnits units
) {
  d->setTurnVelocity(velocity, from_vel(units));
}

// Smartdrive
smartdrive* exp_smartdrive_new(
  motor* l,
  motor* r,
  inertial* i,
  double wheel_travel,
  double track_width,
  double wheel_base,
  double external_gear_ratio
) {
  return new smartdrive(
    *l,
    *r,
    *i,
    wheel_travel,
    track_width,
    wheel_base,
    mm,
    external_gear_ratio
  );
}

void exp_smartdrive_free(smartdrive* s) { delete s; }

void exp_smartdrive_drive(smartdrive* d, Direction dir) {
  d->drive(from_dir(dir));
}

void exp_smartdrive_drive_velocity(
  smartdrive* d,
  Direction dir,
  double velocity,
  VelocityUnits units
) {
  d->drive(from_dir(dir), velocity, from_vel(units));
}

bool exp_smartdrive_drive_for(
  smartdrive* d,
  Direction dir,
  double distance,
  bool wait
) {
  return d->driveFor(from_dir(dir), distance, mm, wait);
}

bool exp_smartdrive_drive_for_velocity(
  smartdrive* d,
  Direction dir,
  double distance,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return d
    ->driveFor(from_dir(dir), distance, mm, velocity, from_vel(units), wait);
}

void exp_smartdrive_turn(smartdrive* d, TurnDirection dir) {
  d->turn(from_dir(dir));
}

void exp_smartdrive_turn_velocity(
  smartdrive* d,
  TurnDirection dir,
  double velocity,
  VelocityUnits units
) {
  d->turn(from_dir(dir), velocity, from_vel(units));
}

bool exp_smartdrive_turn_for(
  smartdrive* d,
  TurnDirection dir,
  double angle,
  bool wait
) {
  return d->turnFor(from_dir(dir), angle, degrees, wait);
}

bool exp_smartdrive_turn_for_velocity(
  smartdrive* d,
  TurnDirection dir,
  double angle,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return d
    ->turnFor(from_dir(dir), angle, degrees, velocity, from_vel(units), wait);
}

void exp_smartdrive_stop(smartdrive* d) { d->stop(); }

void exp_smartdrive_set_drive_velocity(
  smartdrive* d,
  double velocity,
  VelocityUnits units
) {
  d->setDriveVelocity(velocity, from_vel(units));
}

void exp_smartdrive_set_turn_velocity(
  smartdrive* d,
  double velocity,
  VelocityUnits units
) {
  d->setTurnVelocity(velocity, from_vel(units));
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

void exp_motor_spin(motor* m, Direction dir) { m->spin(from_dir(dir)); }

void exp_motor_spin_velocity(
  motor* m,
  Direction dir,
  double velocity,
  VelocityUnits units
) {
  m->spin(from_dir(dir), velocity, from_vel(units));
}

void exp_motor_spin_volts(motor* m, Direction dir, double voltage) {
  m->spin(from_dir(dir), voltage, volt);
}

bool exp_motor_spin_for(motor* m, Direction dir, double rotation, bool wait) {
  return m->spinFor(from_dir(dir), rotation, degrees, wait);
}

bool exp_motor_spin_for_velocity(
  motor* m,
  Direction dir,
  double rotation,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return m->spinFor(
    from_dir(dir),
    rotation,
    degrees,
    velocity,
    from_vel(units),
    wait
  );
}

bool exp_motor_spin_for_time(motor* m, Direction dir, double time) {
  return m->spinFor(from_dir(dir), time, msec);
}

bool exp_motor_spin_for_time_velocity(
  motor* m,
  Direction dir,
  double time,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return m->spinFor(from_dir(dir), time, msec, velocity, from_vel(units));
}

bool exp_motor_spin_to_position(motor* m, double rotation, bool wait) {
  return m->spinToPosition(rotation, degrees, wait);
}

bool exp_motor_spin_to_position_velocity(
  motor* m,
  double rotation,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return m->spinToPosition(rotation, degrees, velocity, from_vel(units), wait);
}

void exp_motor_stop(motor* m) { m->stop(); }

void exp_motor_stop_mode(motor* m, BrakeType mode) {
  m->stop(from_brake(mode));
}

void exp_motor_set_position(motor* m, double value) {
  m->setPosition(value, degrees);
}

void exp_motor_set_velocity(motor* m, double velocity, VelocityUnits units) {
  m->setVelocity(velocity, from_vel(units));
}

void exp_motor_set_stopping(motor* m, BrakeType mode) {
  m->setStopping(from_brake(mode));
}

void exp_motor_set_max_torque(motor* m, double value) {
  m->setMaxTorque(value, Nm);
}

void exp_motor_set_max_torque_current(motor* m, double value) {
  m->setMaxTorque(value, amp);
}

void exp_motor_set_max_torque_percent(motor* m, double value) {
  m->setMaxTorque(value, percent);
}

void exp_motor_set_timeout(motor* m, int32_t time) {
  m->setTimeout(time, msec);
}

bool exp_motor_is_done(motor* m) { return m->isDone(); }

bool exp_motor_is_spinning(motor* m) { return m->isSpinning(); }

double exp_motor_position(motor* m) { return m->position(degrees); }

double exp_motor_velocity(motor* m, VelocityUnits units) {
  return m->velocity(from_vel(units));
}

double exp_motor_current(motor* m) { return m->current(amp); }

double exp_motor_current_percent(motor* m) { return m->current(percent); }

double exp_motor_power(motor* m) { return m->power(watt); }

double exp_motor_torque(motor* m) { return m->torque(Nm); }

double exp_motor_efficiency(motor* m) { return m->efficiency(percent); }

double exp_motor_temperature(motor* m) { return m->temperature(celsius); }

double exp_motor_temperature_percent(motor* m) {
  return m->temperature(percent);
}

double exp_motor_voltage(motor* m) { return m->voltage(volt); }

Direction exp_motor_direction(motor* m) { return to_dir(m->direction()); }

bool exp_motor_installed(motor* m) { return m->installed(); }

// Motor group
motor_group* exp_motor_group_new(motor** motors, size_t len) {
  motor_group* m = new motor_group();
  for (size_t i = 0; i < len; i++) (*m)(*motors[i]);
  return m;
}

void exp_motor_group_free(motor_group* g) { delete g; }

void exp_motor_group_spin(motor_group* m, Direction dir) {
  m->spin(from_dir(dir));
}

void exp_motor_group_spin_velocity(
  motor_group* m,
  Direction dir,
  double velocity,
  VelocityUnits units
) {
  m->spin(from_dir(dir), velocity, from_vel(units));
}

void exp_motor_group_spin_volts(motor_group* m, Direction dir, double voltage) {
  m->spin(from_dir(dir), voltage, volt);
}

bool exp_motor_group_spin_for(
  motor_group* m,
  Direction dir,
  double rotation,
  bool wait
) {
  return m->spinFor(from_dir(dir), rotation, degrees, wait);
}

bool exp_motor_group_spin_for_velocity(
  motor_group* m,
  Direction dir,
  double rotation,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return m->spinFor(
    from_dir(dir),
    rotation,
    degrees,
    velocity,
    from_vel(units),
    wait
  );
}

void exp_motor_group_spin_for_time(motor_group* m, Direction dir, double time) {
  m->spinFor(from_dir(dir), time, msec);
}

void exp_motor_group_spin_for_time_velocity(
  motor_group* m,
  Direction dir,
  double time,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  m->spinFor(from_dir(dir), time, msec, velocity, from_vel(units));
}

bool exp_motor_group_spin_to_position(
  motor_group* m,
  double rotation,
  bool wait
) {
  return m->spinToPosition(rotation, degrees, wait);
}

bool exp_motor_group_spin_to_position_velocity(
  motor_group* m,
  double rotation,
  double velocity,
  VelocityUnits units,
  bool wait
) {
  return m->spinToPosition(rotation, degrees, velocity, from_vel(units), wait);
}

void exp_motor_group_stop(motor_group* m) { m->stop(); }

void exp_motor_group_stop_mode(motor_group* m, BrakeType mode) {
  m->stop(from_brake(mode));
}

void exp_motor_group_set_position(motor_group* m, double value) {
  m->setPosition(value, degrees);
}

void exp_motor_group_set_velocity(
  motor_group* m,
  double velocity,
  VelocityUnits units
) {
  m->setVelocity(velocity, from_vel(units));
}

void exp_motor_group_set_stopping(motor_group* m, BrakeType mode) {
  m->setStopping(from_brake(mode));
}

void exp_motor_group_set_max_torque(motor_group* m, double value) {
  m->setMaxTorque(value, percent);
}

void exp_motor_group_set_max_torque_current(motor_group* m, double value) {
  m->setMaxTorque(value, amp);
}

void exp_motor_group_set_max_torque_percent(motor_group* m, double value) {
  m->setMaxTorque(value, percent);
}

void exp_motor_group_set_timeout(motor_group* m, int32_t time) {
  m->setTimeout(time, msec);
}

bool exp_motor_group_is_done(motor_group* m) { return m->isDone(); }

bool exp_motor_group_is_spinning(motor_group* m) { return m->isSpinning(); }

double exp_motor_group_position(motor_group* m) { return m->position(degrees); }

double exp_motor_group_velocity(motor_group* m, VelocityUnits units) {
  return m->velocity(from_vel(units));
}

double exp_motor_group_current(motor_group* m) { return m->current(amp); }

double exp_motor_group_current_percent(motor_group* m) {
  return m->current(percent);
}

double exp_motor_group_power(motor_group* m) { return m->power(watt); }

double exp_motor_group_torque(motor_group* m) { return m->torque(Nm); }

double exp_motor_group_efficiency(motor_group* m) {
  return m->efficiency(percent);
}

double exp_motor_group_temperature(motor_group* m) {
  return m->temperature(celsius);
}

double exp_motor_group_temperature_percent(motor_group* m) {
  return m->temperature(percent);
}

double exp_motor_group_voltage(motor_group* m) { return m->voltage(volt); }

Direction exp_motor_group_direction(motor_group* m) {
  return to_dir(m->direction());
}

int32_t exp_motor_group_count(motor_group* m) { return m->count(); }

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
