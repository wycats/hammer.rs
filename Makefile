HAMMER_LIB := $(shell rustc --crate-file-name src/hammer.rs --crate-type=rlib)

default: target/$(HAMMER_LIB)

target:
	mkdir -p target

clean:
	rm -rf target

target/$(HAMMER_LIB): target src/hammer.rs
	rustc src/hammer.rs --out-dir target --crate-type=rlib

tests:
	rustc --test src/hammer.rs --out-dir target -o target/tests
	./target/tests

.PHONY: default clean tests
