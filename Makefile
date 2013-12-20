RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTPKG ?= rustpkg
RUSTFLAGS ?= -O -Z debug-info
RUSTLIBFLAGS ?= --dylib --rlib
RUST_REPOSITORY ?= ../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust
VERSION=0.1-pre

codegen_files=\
	        src/codegen/branchify.rs \
	        src/codegen/main.rs \
	        src/codegen/read_method.rs \
	        src/codegen/status.rs \

libhttp_so=build/libhttp-9296ff29-0.1-pre.so
http_files=\
		      src/http/lib.rs \
		      src/http/buffer.rs \
		      src/http/common.rs \
		      src/http/generated/read_method.rs \
		      src/http/generated/status.rs \
		      $(wildcard src/http/headers/*.rs) \
		      $(wildcard src/http/client/*.rs) \
		      $(wildcard src/http/server/*.rs) \
		      src/http/memstream.rs \
		      src/http/method.rs \
		      src/http/rfc2616.rs

http: $(libhttp_so)

$(libhttp_so): $(http_files)
	mkdir -p build/
	$(RUSTC) $(RUSTFLAGS) $(RUSTLIBFLAGS) src/http/lib.rs --out-dir=build

all: http examples docs

build/codegen: $(codegen_files)
	mkdir -p build/
	$(RUSTC) $(RUSTFLAGS) src/codegen/main.rs --out-dir=build

src/http/generated/%.rs: build/codegen
	build/codegen $(patsubst src/http/generated/%,%,$@) src/http/generated/

build/%:: src/%/main.rs $(libhttp_so)
	mkdir -p "$(dir $@)"
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L build/

examples: $(patsubst src/examples/%/main.rs,build/examples/%,$(wildcard src/examples/*/main.rs)) \
		  $(patsubst src/examples/%/main.rs,build/examples/%,$(wildcard src/examples/*/*/main.rs))

docs: doc/http/index.html

doc/http/index.html: $(http_files)
	$(RUSTDOC) src/http/lib.rs

build/tests: $(http_files)
	$(RUSTC) $(RUSTFLAGS) --test -o build/tests src/http/lib.rs

build/quicktests: $(http_files)
	$(RUSTC) --test -o build/quicktests src/http/lib.rs

# Can't wait for everything to build, optimised too? OK, you can save some time here.
quickcheck: build/quicktests
	build/quicktests --test

check: all build/tests
	build/tests --test

clean:
	rm -rf src/http/generated/ src/http/codegen/codegen
	rm -rf build/
	rm -rf bin/ .rust/

TAGS:
	ctags -f TAGS --options=$(RUST_CTAGS) -R src

.PHONY: all http examples docs clean check quickcheck
