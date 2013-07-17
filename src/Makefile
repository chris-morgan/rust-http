RUST=rust
RUSTC=rustc
RUSTFLAGS=-Z debug-info -O
RUST_NEWRT=0

RUST_NEWRT=1
RUSTFLAGS=-Z debug-info -O --cfg newrt

lrhs_so=librusthttpserver/librusthttpserver-20af9b1d3441fe5a-0.1-pre.so
.PHONY: librusthttpserver

librusthttpserver: $(lrhs_so)

librusthttpserver/status.rs: librusthttpserver/generate_status.py
	cd librusthttpserver; python2 generate_status.py


$(lrhs_so): librusthttpserver/adapter.rs librusthttpserver/headers.rs librusthttpserver/methods.rs librusthttpserver/mod.rs librusthttpserver/request.rs librusthttpserver/response.rs librusthttpserver/rfc2616.rs librusthttpserver/rusthttpserver.rs librusthttpserver/server.rs librusthttpserver/status.rs
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) rusthttpserver.rs

%: %.rs librusthttpserver
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L librusthttpserver

run-%: %
	if [ "$(RUST_NEWRT)" = 0 ]; then $<; else RUST_NEWRT=$(RUST_NEWRT) $<; fi

test-%: %.rs
	$(RUST) test $(RUSTFLAGS) $<
