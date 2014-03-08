HAMMER_LIB := $(shell rustc --crate-file-name src/lib.rs --crate-type=rlib)

default: target/$(HAMMER_LIB)

target:
	mkdir -p target

clean:
	rm -rf target

target/$(HAMMER_LIB): target src/lib.rs
	rustc src/lib.rs --out-dir target --crate-type=rlib
