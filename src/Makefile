RUST=rust
RUSTC=rustc
#RUSTFLAGS=-Z debug-info -O
RUSTFLAGS=-O
VERSION=0.1-pre

lrhs_so=librusthttpserver/librusthttpserver-20af9b1d3441fe5a-$(VERSION).so
lrhs_files=librusthttpserver/buffer.rs librusthttpserver/headers.rs librusthttpserver/method.rs librusthttpserver/request.rs librusthttpserver/response.rs librusthttpserver/rfc2616.rs librusthttpserver/rusthttpserver.rs librusthttpserver/server.rs librusthttpserver/status.rs

all: $(lrhs_so) examples

librusthttpserver/status.rs: librusthttpserver/generate_status.py
	cd librusthttpserver; python2 generate_status.py

$(lrhs_so): $(lrhs_files)
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) rusthttpserver.rs

%:: %.rs $(lrhs_so)
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L librusthttpserver

run-%: %
	RUST_NEWRT=1 $<

test-%: %.rs
	RUST_NEWRT=1 $(RUST) test $(RUSTFLAGS) $<

examples: examples/apache_fake examples/hello_world examples/info

tests: $(lrhs_files)
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) --test rusthttpserver.rs
	librusthttpserver/rusthttpserver --test

clean-tests:
	rm -f librusthttpserver/rusthttpserver

clean: clean-tests
	rm -f $(lrhs_so)
	rm -f examples/apache_fake examples/hello_world examples/info

.PHONY: all examples clean tests clean-tests
