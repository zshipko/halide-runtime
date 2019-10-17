HALIDE_PATH?=~/halide

bindings:
	bindgen \
		--raw-line '#![allow(warnings)]' \
		--rustified-enum "halide.*_t" \
		--whitelist-type 'halide.*_t' \
		--whitelist-function 'halide.*' \
		--no-doc-comments \
		$(HALIDE_PATH)/src/runtime/HalideRuntime.h > src/runtime.rs

test:
	halide-build run brighter.cpp
	$(CC) -shared -o libbrighter.so brighter.o
	cargo test
