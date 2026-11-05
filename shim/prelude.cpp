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

velocityUnits to_vex_enum(VelocityUnits in) {
  if (in == VelocityUnits::Percent) {
    return velocityUnits::pct;
  }
  return velocityUnits::rpm;
}
}  // namespace
