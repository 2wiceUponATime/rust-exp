#include "exp_cpp.h"
#include <stdio.h>

#define PORT_CASE(i, c)                                                        \
  case i:                                                                      \
    return new c(PORT##i)
#define PORT_CASES(c)                                                          \
  PORT_CASE(1, c);                                                             \
  PORT_CASE(2, c);                                                             \
  PORT_CASE(3, c);                                                             \
  PORT_CASE(4, c);                                                             \
  PORT_CASE(5, c);                                                             \
  PORT_CASE(6, c);                                                             \
  PORT_CASE(7, c);                                                             \
  PORT_CASE(8, c);                                                             \
  PORT_CASE(9, c);                                                             \
  PORT_CASE(10, c);                                                            \
  PORT_CASE(11, c);                                                            \
  PORT_CASE(12, c);                                                            \
  PORT_CASE(13, c);                                                            \
  PORT_CASE(14, c);                                                            \
  PORT_CASE(15, c);                                                            \
  PORT_CASE(16, c);                                                            \
  PORT_CASE(17, c);                                                            \
  PORT_CASE(18, c);                                                            \
  PORT_CASE(19, c);                                                            \
  PORT_CASE(20, c);                                                            \
  PORT_CASE(21, c);                                                            \
  PORT_CASE(22, c)

using namespace vex;

extern "C" {
// Stdio
void exp_printf(char *s) { printf("%s", s); }

int exp_flush_stdout() { return fflush(stdout); }

// Brain
brain *exp_brain_new() { return new brain; }

void exp_brain_free(brain *b) { delete b; }

void exp_brain_print_at(brain *brain, int32_t x, int32_t y, char *s) {
  brain->Screen.printAt(x, y, s);
}

// Motor
motor *exp_motor_new(uint8_t port) {
  switch (port) {
    PORT_CASES(motor);
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

// Smartdrive
smartdrive *exp_smartdrive_new(motor *l, motor *r, inertial *i) {
  return new smartdrive(*l, *r, *i);
}

void exp_smartdrive_free(smartdrive *s) { delete s; }

void exp_smartdrive_turn_to_heading(smartdrive *s, double angle) {
  s->turnToHeading(angle, degrees);
}
}
