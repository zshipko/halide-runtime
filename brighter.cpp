#define IMAGED_HALIDE_UTIL
#include <imaged.h>
#include <stdio.h>

class Brighter : public Generator<Brighter> {
public:
  Input<Buffer<uint8_t>> input{"input", 3};
  Output<Buffer<uint8_t>> brighter{"brighter", 3};

  Var x, y, c;

  void generate() {
    brighter(x, y, c) = input(x, y, c) + 10;
    interleave_input(input, 3, x, y, c);
    interleave_output(brighter, 3, x, y, c);
  }
};

HALIDE_REGISTER_GENERATOR(Brighter, brighter);
