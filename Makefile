RUSTC ?= rustc
HAMMER_LIB := $(shell $(RUSTC) --crate-file-name src/hammer.rs --crate-type=rlib)

default: target/$(HAMMER_LIB)

target:
	mkdir -p target

clean:
	rm -rf target

target/$(HAMMER_LIB): target src/*.rs
	$(RUSTC) src/hammer.rs --out-dir target --crate-type=rlib

tests:
	$(RUSTC) --test src/hammer.rs -o target/tests
	./target/tests

.PHONY: default clean tests
