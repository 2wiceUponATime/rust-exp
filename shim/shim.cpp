#include "exp_cpp.h"

using namespace vex;

extern "C" {
  brain* exp_brain_new() {
    return new vex::brain;
  }

  void exp_brain_free(brain* b) {
    delete b;
  }

  void exp_brain_print_at(brain* brain, int32_t x, int32_t y, const char* text) {
    brain->Screen.printAt(x, y, text);
  }
}
