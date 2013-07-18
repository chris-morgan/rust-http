RUST=rust
RUSTC=rustc
RUSTFLAGS_BASE=-Z debug-info -O
RUST_NEWRT=1
VERSION=0.1-pre

ifeq ($(RUST_NEWRT),1)
RUSTFLAGS=$(RUSTFLAGS_BASE) --cfg newrt
else
RUSTFLAGS=$(RUSTFLAGS_BASE)
endif

lrhs_so=librusthttpserver/librusthttpserver-20af9b1d3441fe5a-$(VERSION).so

librusthttpserver/status.rs: librusthttpserver/generate_status.py
	cd librusthttpserver; python2 generate_status.py


$(lrhs_so): librusthttpserver/adapter.rs librusthttpserver/headers.rs librusthttpserver/methods.rs librusthttpserver/mod.rs librusthttpserver/request.rs librusthttpserver/response.rs librusthttpserver/rfc2616.rs librusthttpserver/rusthttpserver.rs librusthttpserver/server.rs librusthttpserver/status.rs
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) rusthttpserver.rs

%:: %.rs $(lrhs_so)
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L librusthttpserver

ifeq ($(RUST_NEWRT),1)
run-%: %
	RUST_NEWRT=$(RUST_NEWRT) $<
else
run-%: %
	$<
endif

test-%: %.rs
	$(RUST) test $(RUSTFLAGS) $<

clean:
	rm -f $(lrhs_so) ws?

.PHONY: clean
