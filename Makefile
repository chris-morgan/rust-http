RUSTC ?= rustc
RUSTPKG ?= rustpkg
RUSTFLAGS ?= -O -Z debug-info

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

all: http examples

src/http/generated:
	mkdir -p src/http/generated

src/http/generated/%.rs: bin/codegen src/http/generated
	./bin/codegen $(patsubst src/http/generated/%,%,$@) src/http/generated/

http: $(http_files)
	$(RUSTPKG) install $(RUSTFLAGS) http

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

examples: bin/apache_fake bin/apache_fake bin/hello_world bin/info bin/request_uri

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
	rm -rf bin build lib src/http/generated

.PHONY: all http examples clean check quickcheck
