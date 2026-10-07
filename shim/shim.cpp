#include "exp_cpp.h"
#include <stdio.h>

#define PORT_CASE(i, c, ...)                                                   \
  case i:                                                                      \
    return new c(PORT##i __VA_OPT__(, ) __VA_ARGS__)
#define PORT_CASES(...)                                                        \
  PORT_CASE(1, __VA_ARGS__);                                                   \
  PORT_CASE(2, __VA_ARGS__);                                                   \
  PORT_CASE(3, __VA_ARGS__);                                                   \
  PORT_CASE(4, __VA_ARGS__);                                                   \
  PORT_CASE(5, __VA_ARGS__);                                                   \
  PORT_CASE(6, __VA_ARGS__);                                                   \
  PORT_CASE(7, __VA_ARGS__);                                                   \
  PORT_CASE(8, __VA_ARGS__);                                                   \
  PORT_CASE(9, __VA_ARGS__);                                                   \
  PORT_CASE(10, __VA_ARGS__);                                                  \
  PORT_CASE(11, __VA_ARGS__);                                                  \
  PORT_CASE(12, __VA_ARGS__);                                                  \
  PORT_CASE(13, __VA_ARGS__);                                                  \
  PORT_CASE(14, __VA_ARGS__);                                                  \
  PORT_CASE(15, __VA_ARGS__);                                                  \
  PORT_CASE(16, __VA_ARGS__);                                                  \
  PORT_CASE(17, __VA_ARGS__);                                                  \
  PORT_CASE(18, __VA_ARGS__);                                                  \
  PORT_CASE(19, __VA_ARGS__);                                                  \
  PORT_CASE(20, __VA_ARGS__);                                                  \
  PORT_CASE(21, __VA_ARGS__);                                                  \
  PORT_CASE(22, __VA_ARGS__)

using namespace vex;

enum class DriveDirection : uint8_t {
  Forward = 0,
  Reverse = 1,
};

enum class TurnDirection : uint8_t {
  Left = 0,
  Right = 1,
};

extern "C" {
// Stdio
void exp_printf(char *s) { printf("%s", s); }

int exp_flush_stdout() { return fflush(stdout); }

// Threads
void exp_thread_sleep(uint32_t time) { this_thread::sleep_for(time); }

// Brain
brain *exp_brain_new() { return new brain; }

void exp_brain_free(brain *b) { delete b; }

void exp_brain_screen_clear(brain *brain) { brain->Screen.clearScreen(); }

void exp_brain_screen_print_at(brain *brain, int32_t x, int32_t y, char *s) {
  brain->Screen.printAt(x, y, s);
}

// Motor
motor *exp_motor_new(uint8_t port, bool reverse) {
  switch (port) {
    PORT_CASES(motor, reverse);
  default:
    return 0;
  }
}

void exp_motor_free(motor *m) { delete m; }

// Inertial
inertial *exp_inertial_new() { return new inertial(); }

inertial *exp_inertial_new_port(uint8_t port) {
  switch (port) {
    PORT_CASES(inertial);
  default:
    return 0;
  }
}

void exp_inertial_free(inertial *i) { delete i; }

void exp_inertial_calibrate(inertial *i) { i->calibrate(); }

bool exp_inertial_is_calibrating(inertial *i) { return i->isCalibrating(); }

double exp_inertial_angle(inertial *i) { return i->angle(degrees); }

// Smartdrive
smartdrive *exp_smartdrive_new(motor *l, motor *r, inertial *i) {
  return new smartdrive(*l, *r, *i);
}

void exp_smartdrive_free(smartdrive *s) { delete s; }

void exp_smartdrive_set_drive_velocity(smartdrive *s, double velocity) {
  s->setDriveVelocity(velocity, percent);
}

void exp_smartdrive_set_turn_velocity(smartdrive *s, double velocity) {
  s->setTurnVelocity(velocity, percent);
}

void exp_smartdrive_turn_to_heading(smartdrive *s, double angle) {
  s->turnToHeading(angle, degrees);
}

void exp_smartdrive_drive(smartdrive *s, DriveDirection dir) {
  s->drive(dir == DriveDirection::Forward ? forward : reverse);
}

void exp_smartdrive_turn(smartdrive *s, TurnDirection dir) {
  s->turn(dir == TurnDirection::Left ? left : right);
}

void exp_smartdrive_stop(smartdrive *s) {
  s->stop();
}
}
