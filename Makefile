HALIDE_PATH?=~/halide

HEADER?=HalideRuntime.h

test:
	TEST=1 cargo test

update-header:
	curl -O https://raw.githubusercontent.com/halide/Halide/master/src/runtime/HalideRuntime.h
