#include <stdio.h>

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

velocityUnits to_vex(VelocityUnits in) {
  if (in == VelocityUnits::Percent) {
    return velocityUnits::pct;
  }
  return velocityUnits::rpm;
}

fontType to_vex(FontType in) {
  switch (in) {
    case FontType::Mono20:
      return fontType::mono20;
    case FontType::Mono30:
      return fontType::mono30;
    case FontType::Mono40:
      return fontType::mono40;
    case FontType::Mono60:
      return fontType::mono60;
    case FontType::Prop20:
      return fontType::prop20;
    case FontType::Prop30:
      return fontType::prop30;
    case FontType::Prop40:
      return fontType::prop40;
    case FontType::Prop60:
      return fontType::prop60;
    case FontType::Mono15:
      return fontType::mono15;
    case FontType::Mono12:
      return fontType::mono12;
    default:
      return fontType::cjk16;
  }
}

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
}  // namespace
