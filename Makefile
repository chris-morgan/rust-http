RUST=rust
RUSTC=rustc
#RUSTFLAGS=-Z debug-info -O
RUSTFLAGS=-O
VERSION=0.1-pre

lrhs_so=build/librusthttpserver-20af9b1d3441fe5a-$(VERSION).so
lrhs_files=\
		   src/librusthttpserver/buffer.rs \
		   src/librusthttpserver/generated/read_method.rs \
		   src/librusthttpserver/generated/status.rs \
		   src/librusthttpserver/headers.rs \
		   src/librusthttpserver/method.rs \
		   src/librusthttpserver/request.rs \
		   src/librusthttpserver/response.rs \
		   src/librusthttpserver/rfc2616.rs \
		   src/librusthttpserver/rusthttpserver.rs \
		   src/librusthttpserver/server.rs

all: $(lrhs_so) examples

src/librusthttpserver/codegen/codegen: $(wildcard src/librusthttpserver/codegen/*.rs)
	$(RUSTC) $(RUSTFLAGS) $@.rs

src/librusthttpserver/generated/%.rs: src/librusthttpserver/codegen/codegen
	src/librusthttpserver/codegen/codegen $(patsubst src/librusthttpserver/generated/%,%,$@)

$(lrhs_so): $(lrhs_files)
	mkdir -p build/
	$(RUSTC) $(RUSTFLAGS) src/librusthttpserver/rusthttpserver.rs --out-dir=build

build/%:: src/%.rs $(lrhs_so)
	mkdir -p '$(dir $@)'
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L build/

examples: build/examples/apache_fake build/examples/hello_world build/examples/info

tests: $(lrhs_files)
	$(RUSTC) $(RUSTFLAGS) --test --out-dir=build src/librusthttpserver/rusthttpserver.rs
	build/rusthttpserver --test

clean-tests:
	rm -f build/rusthttpserver

clean: clean-tests
	rm -rf src/librusthttpserver/generated/ src/librusthttpserver/codegen/codegen
	rm -rf build/

.PHONY: all examples clean tests clean-tests
