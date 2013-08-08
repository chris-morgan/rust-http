RUST=rust
RUSTC=rustc
#RUSTFLAGS=-Z debug-info -O
RUSTFLAGS=-O
VERSION=0.1-pre

lrhs_so=librusthttpserver/librusthttpserver-20af9b1d3441fe5a-$(VERSION).so
lrhs_files=\
		   librusthttpserver/buffer.rs \
		   librusthttpserver/generated/read_method.rs \
		   librusthttpserver/generated/status.rs \
		   librusthttpserver/headers.rs \
		   librusthttpserver/method.rs \
		   librusthttpserver/request.rs \
		   librusthttpserver/response.rs \
		   librusthttpserver/rfc2616.rs \
		   librusthttpserver/rusthttpserver.rs \
		   librusthttpserver/server.rs

all: $(lrhs_so) examples

librusthttpserver/codegen/codegen: $(wildcard librusthttpserver/codegen/*.rs)
	$(RUSTC) $(RUSTFLAGS) $@.rs

librusthttpserver/generated/%.rs: librusthttpserver/codegen/codegen
	librusthttpserver/codegen/codegen $(patsubst librusthttpserver/generated/%,%,$@)

$(lrhs_so): $(lrhs_files)
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) rusthttpserver.rs

%:: %.rs $(lrhs_so)
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L librusthttpserver

examples: examples/apache_fake examples/hello_world examples/info

tests: $(lrhs_files)
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) --test rusthttpserver.rs
	librusthttpserver/rusthttpserver --test

clean-tests:
	rm -f librusthttpserver/rusthttpserver

clean: clean-tests
	rm -f $(lrhs_so)
	rm -rf librusthttpserver/generated/ librusthttpserver/codegen/codegen
	rm -f examples/apache_fake examples/hello_world examples/info

.PHONY: all examples clean tests clean-tests
