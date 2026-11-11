#include <errno.h>
#include <stdio.h>

#include <cstdint>

#include "exp_cpp.h"

using namespace vex;

#define ASSERT_SAME(a, b)                                                      \
  static_assert(static_cast<int>(a) == static_cast<int>(b), #a " != " #b)
#define AXIS_CASE(variant)                                                     \
  case AxisType::variant:                                                      \
    return const_cast<controller::axis *>(&c->variant);
#define BUTTON_CASE(variant)                                                   \
  case ButtonType::variant:                                                    \
    return const_cast<controller::button *>(&c->Button##variant);

namespace {

template <typename F> using fn_ptr = F *;

struct Color {
  uint8_t r;
  uint8_t g;
  uint8_t b;
  bool transparent;
};

enum class AxisType : uint8_t {
  Axis3 = 0,
  Axis4,
  Axis1,
  Axis2,
};

enum class BrakeType : uint8_t {
  Coast = 0,
  Brake,
  Hold,
};

ASSERT_SAME(BrakeType::Coast, brakeType::coast);
ASSERT_SAME(BrakeType::Brake, brakeType::brake);
ASSERT_SAME(BrakeType::Hold, brakeType::hold);

enum class ButtonType : uint8_t {
  L1 = 0,
  L2,
  R1,
  R2,
  Up,
  Down,
  Left,
  Right,
  X,
  B,
  Y,
  A,
  L3,
  R3,
};

enum class ControllerType : uint8_t {
  Primary = 0,
  Partner,
};

ASSERT_SAME(ControllerType::Primary, controllerType::primary);
ASSERT_SAME(ControllerType::Partner, controllerType::partner);

enum class Direction : uint8_t {
  Forward,
  Reverse,
};

ASSERT_SAME(Direction::Forward, directionType::fwd);
ASSERT_SAME(Direction::Reverse, directionType::rev);

enum class FontType : uint8_t {
  Mono20 = 0,
  Mono30,
  Mono40,
  Mono60,
  Prop20,
  Prop30,
  Prop40,
  Prop60,
  Mono15,
  Mono12,
  Cjk16,
};

ASSERT_SAME(FontType::Mono20, fontType::mono20);
ASSERT_SAME(FontType::Mono30, fontType::mono30);
ASSERT_SAME(FontType::Mono40, fontType::mono40);
ASSERT_SAME(FontType::Mono60, fontType::mono60);
ASSERT_SAME(FontType::Prop20, fontType::prop20);
ASSERT_SAME(FontType::Prop30, fontType::prop30);
ASSERT_SAME(FontType::Prop40, fontType::prop40);
ASSERT_SAME(FontType::Prop60, fontType::prop60);
ASSERT_SAME(FontType::Mono15, fontType::mono15);
ASSERT_SAME(FontType::Mono12, fontType::mono12);
ASSERT_SAME(FontType::Cjk16, fontType::cjk16);

enum class FResult : uint8_t {
  Ok = 0,
  DiskErr,
  IntErr,
  NotReady,
  NoFile,
  NoPath,
  InvalidName,
  Denied,
  Exist,
  InvalidObject,
  WriteProtected,
  InvalidDrive,
  NotEnabled,
  NoFilesystem,
  MkfsAborted,
  Timeout,
  Locked,
  NotEnoughCore,
  TooManyOpenFiles,
  InvalidParameter,
};

ASSERT_SAME(FResult::Ok, FR_OK);
ASSERT_SAME(FResult::DiskErr, FR_DISK_ERR);
ASSERT_SAME(FResult::IntErr, FR_INT_ERR);
ASSERT_SAME(FResult::NotReady, FR_NOT_READY);
ASSERT_SAME(FResult::NoFile, FR_NO_FILE);
ASSERT_SAME(FResult::NoPath, FR_NO_PATH);
ASSERT_SAME(FResult::InvalidName, FR_INVALID_NAME);
ASSERT_SAME(FResult::Denied, FR_DENIED);
ASSERT_SAME(FResult::Exist, FR_EXIST);
ASSERT_SAME(FResult::InvalidObject, FR_INVALID_OBJECT);
ASSERT_SAME(FResult::WriteProtected, FR_WRITE_PROTECTED);
ASSERT_SAME(FResult::InvalidDrive, FR_INVALID_DRIVE);
ASSERT_SAME(FResult::NotEnabled, FR_NOT_ENABLED);
ASSERT_SAME(FResult::NoFilesystem, FR_NO_FILESYSTEM);
ASSERT_SAME(FResult::MkfsAborted, FR_MKFS_ABORTED);
ASSERT_SAME(FResult::Timeout, FR_TIMEOUT);
ASSERT_SAME(FResult::Locked, FR_LOCKED);
ASSERT_SAME(FResult::NotEnoughCore, FR_NOT_ENOUGH_CORE);
ASSERT_SAME(FResult::TooManyOpenFiles, FR_TOO_MANY_OPEN_FILES);
ASSERT_SAME(FResult::InvalidParameter, FR_INVALID_PARAMETER);

enum class GearSetting : uint8_t { R36_1 = 0, R18_1, R6_1 };

ASSERT_SAME(GearSetting::R36_1, gearSetting::ratio36_1);
ASSERT_SAME(GearSetting::R18_1, gearSetting::ratio18_1);
ASSERT_SAME(GearSetting::R6_1, gearSetting::ratio6_1);

enum class TurnDirection : uint8_t {
  Left = 0,
  Right,
};

ASSERT_SAME(TurnDirection::Left, turnType::left);
ASSERT_SAME(TurnDirection::Right, turnType::right);

enum class VelocityUnits : uint8_t {
  Percent = 0,
  Rpm,
};

ASSERT_SAME(VelocityUnits::Percent, velocityUnits::pct);
ASSERT_SAME(VelocityUnits::Rpm, velocityUnits::rpm);

enum class Whence : uint8_t {
  Start = 0,
  Current,
  End,
};

ASSERT_SAME(Whence::Start, SEEK_SET);
ASSERT_SAME(Whence::Current, SEEK_CUR);
ASSERT_SAME(Whence::End, SEEK_END);

color to_vex(Color in) {
  return in.transparent ? color::transparent : color(in.r, in.g, in.b);
}

brakeType to_vex(BrakeType in) { return static_cast<brakeType>(in); }

controllerType to_vex(ControllerType in) {
  return static_cast<controllerType>(in);
}

directionType to_vex(Direction in) { return static_cast<directionType>(in); }

fontType to_vex(FontType in) { return static_cast<fontType>(in); }

gearSetting to_vex(GearSetting in) { return static_cast<gearSetting>(in); }

turnType to_vex(TurnDirection in) { return static_cast<turnType>(in); }

velocityUnits to_vex(VelocityUnits in) {
  return static_cast<velocityUnits>(in);
}

int32_t to_vex(Whence in) { return static_cast<int32_t>(in); }

GearSetting from_vex(gearSetting out) { return static_cast<GearSetting>(out); }

FResult from_vex(FRESULT out) { return static_cast<FResult>(out); }

motor_group *group_from_motors(motor **motors, size_t motor_count) {
  auto *g = new motor_group();
  for (size_t i = 0; i < motor_count; i++) {
    (*g)(*motors[i]);
  }
  return g;
}

controller::button *get_controller_button(controller *c, ButtonType type) {
  switch (type) {
    BUTTON_CASE(L1);
    BUTTON_CASE(L2);
    BUTTON_CASE(R1);
    BUTTON_CASE(R2);
    BUTTON_CASE(Up);
    BUTTON_CASE(Down);
    BUTTON_CASE(Left);
    BUTTON_CASE(Right);
    BUTTON_CASE(X);
    BUTTON_CASE(B);
    BUTTON_CASE(Y);
    BUTTON_CASE(A);
    BUTTON_CASE(L3);
    BUTTON_CASE(R3);
  }
}

controller::axis *get_controller_axis(controller *c, AxisType type) {
  switch (type) {
    AXIS_CASE(Axis1);
    AXIS_CASE(Axis2);
    AXIS_CASE(Axis3);
    AXIS_CASE(Axis4);
  }
}
} // namespace
