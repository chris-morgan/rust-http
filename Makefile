RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTPKG ?= rustpkg
RUSTFLAGS ?= -O -Z debug-info
RUST_REPOSITORY ?= ../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust
VERSION=0.1-pre

codegen_files=\
	        src/codegen/branchify.rs \
	        src/codegen/main.rs \
	        src/codegen/read_method.rs \
	        src/codegen/status.rs \

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

http: $(http_files)
	$(RUSTPKG) install $(RUSTFLAGS) http

all: http examples docs

src/http/generated:
	mkdir -p src/http/generated

src/http/generated/%.rs: bin/codegen src/http/generated
	./bin/codegen $(patsubst src/http/generated/%,%,$@) src/http/generated/

bin/codegen: $(codegen_files)
	$(RUSTPKG) install $(RUSTFLAGS) codegen

bin/client: http src/examples/client/main.rs
	$(RUSTPKG) install $(RUSTFLAGS) examples/client

bin/apache_fake: http src/examples/server/apache_fake/main.rs
	$(RUSTPKG) install $(RUSTFLAGS) examples/server/apache_fake

bin/hello_world: http src/examples/server/hello_world/main.rs
	$(RUSTPKG) install $(RUSTFLAGS) examples/server/hello_world

bin/info: http src/examples/server/info/main.rs
	$(RUSTPKG) install $(RUSTFLAGS) examples/server/info

bin/request_uri: http src/examples/server/request_uri/main.rs
	$(RUSTPKG) install $(RUSTFLAGS) examples/server/request_uri

examples: bin/client bin/apache_fake bin/hello_world bin/info bin/request_uri

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
	rm -rf bin build lib src/http/generated .rust

TAGS:
	ctags -f TAGS --options=$(RUST_CTAGS) -R src

.PHONY: all http examples docs clean check quickcheck
