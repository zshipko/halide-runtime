#include <stdio.h>

#include <Halide.h>

using namespace Halide;

template <typename T>
void interleave_input(T &input, Expr n, Var x, Var y, Var c) {
  input.dim(0).set_stride(n).dim(2).set_stride(1);
  input.dim(2).set_bounds(0, n);
}

template <typename T>
void interleave_output(T &output, Expr n, Var x, Var y, Var c) {
  output.dim(0).set_stride(n).dim(2).set_stride(1);
  output.dim(2).set_bounds(0, n);
  output.reorder(c, x, y).unroll(c);
}

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
