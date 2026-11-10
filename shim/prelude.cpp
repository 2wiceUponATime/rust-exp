#include <errno.h>
#include <exp_apitypes.h>
#include <stdio.h>

#include <cstdint>

#include "exp_cpp.h"

using namespace vex;

namespace {
template <typename F>
using fn_ptr = F*;

enum class VelocityUnits : uint8_t {
  Percent = 0,
  Rpm,
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

enum class Whence : uint8_t {
  Start = 0,
  Current,
  End,
};

fontType to_vex(FontType in) { return static_cast<fontType>(in); }

color to_vex(Color in) {
  return in.transparent ? color::transparent : color(in.r, in.g, in.b);
}

int32_t to_vex(Whence in) { return static_cast<int32_t>(in); }

FResult from_vex(FRESULT out) { return static_cast<FResult>(out); }
}  // namespace
