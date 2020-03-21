HALIDE_PATH?=~/halide

HEADER?=HalideRuntime.h

build: bindings
	cargo build

bindings:
	bindgen \
		--raw-line '#![allow(warnings)]' \
		--rustified-enum "halide.*_t" \
		--whitelist-type 'halide.*_t' \
		--whitelist-function 'halide.*' \
		--no-doc-comments \
	$(HEADER) > src/runtime.rs

test:
	halide run -g brighter.cpp -- -g brighter -o . -e o target=host
	$(CC) -shared -o libbrighter.so brighter.o
	cargo test

update-header:
	curl -O https://raw.githubusercontent.com/halide/Halide/master/src/runtime/HalideRuntime.h
