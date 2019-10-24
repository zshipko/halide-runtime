HALIDE_PATH?=~/halide

build: bindings
	cargo build

bindings:
	bindgen \
		--raw-line '#![allow(warnings)]' \
		--rustified-enum "halide.*_t" \
		--whitelist-type 'halide.*_t' \
		--whitelist-function 'halide.*' \
		--no-doc-comments \
	HalideRuntime.h > src/runtime.rs

test:
	halide-build run -g brighter.cpp -- -g brighter -o . -e o target=host
	$(CC) -shared -o libbrighter.so brighter.o
	cargo test

update-header:
	curl -O https://raw.githubusercontent.com/halide/Halide/master/src/runtime/HalideRuntime.h
