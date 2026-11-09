#include <errno.h>
#include <exp_apitypes.h>
#include <stdio.h>
#include <vex_units.h>

#include <cstdint>

#include "exp_cpp.h"

using namespace vex;

namespace {
template <typename F>
using fn_ptr = F*;

enum class VelocityUnits : uint8_t {
  Percent = 0,
  Rpm = 1,
};

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

struct Color {
  uint8_t r;
  uint8_t g;
  uint8_t b;
  bool transparent;
};

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

velocityUnits to_vex(VelocityUnits in) {
  if (in == VelocityUnits::Percent) {
    return velocityUnits::pct;
  }
  return velocityUnits::rpm;
}

fontType to_vex(FontType in) { return static_cast<fontType>(in); }

color to_vex(Color in) {
  return in.transparent ? color::transparent : color(in.r, in.g, in.b);
}

Color from_vex(color out) {
  uint32_t value = out.rgb();
  uint8_t r = value >> 16;
  uint8_t g = value >> 8;
  uint8_t b = value;
  return Color{.r = r, .g = g, .b = b, .transparent = out.isTransparent()};
}

FResult from_vex(FRESULT in) { return static_cast<FResult>(in); }
}  // namespace
