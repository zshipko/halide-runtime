package halide

// #include "../HalideRuntime.h"
/*
struct halide_type_t make_type(int kind, int bits) {
	struct halide_type_t t;
	t.code = kind;
	t.bits = bits;
	return t;
}
*/
import "C"

import (
	"unsafe"
)

type Buffer struct {
	dim    []C.halide_dimension_t
	handle C.halide_buffer_t
}

type Kind int

const (
	Int Kind = iota
	UInt
	Float
)

type Type struct {
	Kind Kind
	Bits int
}

var (
	U8 = Type{
		UInt,
		8,
	}
	U16 = Type{
		UInt,
		16,
	}
	F32 = Type{
		Float,
		32,
	}
)

func (b *Buffer) Ptr() *C.halide_buffer_t {
	return &b.handle
}

func NewBuffer(width, height, channels int, t Type, data unsafe.Pointer) *Buffer {
	var buf C.halide_buffer_t
	buf.flags = 0

	if channels > 1 {
		buf.dimensions = 3
	} else {
		buf.dimensions = 2
	}

	dim := []C.halide_dimension_t{}

	if buf.dimensions == 3 {
		dim = append(dim, C.halide_dimension_t{
			extent: C.int(channels),
			stride: C.int(1),
		})
	}

	dim = append(dim, C.halide_dimension_t{
		extent: C.int(width),
		stride: C.int(channels),
	})
	dim = append(dim, C.halide_dimension_t{
		extent: C.int(height),
		stride: C.int(channels * width),
	})

	buf.host = (*C.uchar)(data)
	buf._type = C.make_type(
		C.int(t.Kind),
		C.int(t.Bits),
	)
	buf.dim = &dim[0]

	return &Buffer{
		dim,
		buf,
	}
}
