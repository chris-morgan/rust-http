RUST=rust
RUSTC=rustc
RUSTFLAGS=-Z debug-info -O
RUST_NEWRT=0

RUST_NEWRT=1
RUSTFLAGS=-Z debug-info -O --cfg newrt

.PHONY: librusthttpserver

librusthttpserver: librusthttpserver-20af9b1d3441fe5a-0.1-pre.so

librusthttpserver-20af9b1d3441fe5a-0.1-pre.so:
	cd librusthttpserver; $(RUSTC) $(RUSTFLAGS) rusthttpserver.rs

%: %.rs librusthttpserver
	$(RUSTC) $(RUSTFLAGS) $< -o $@ -L librusthttpserver

run-%: %
	if [ "$(RUST_NEWRT)" = 0 ]; then $<; else RUST_NEWRT=$(RUST_NEWRT) $<; fi

test-%: %.rs
	$(RUST) test $(RUSTFLAGS) $<
