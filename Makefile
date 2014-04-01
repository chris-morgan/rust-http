RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -O
RUST_REPOSITORY ?= ../rust
RUST_CTAGS ?= $(RUST_REPOSITORY)/src/etc/ctags.rust

libhttp := $(addprefix build/,$(shell $(RUSTC) --crate-file-name src/http/lib.rs))

all: http examples docs

http: $(libhttp)
examples: $(patsubst src/%/main.rs,build/%,$(shell find src/examples -name main.rs))
docs: doc/http/index.html

-include $(wildcard build/*.d build/*/*.d build/*/*/*.d)

src/http/lib.rs: src/http/generated/read_method.rs src/http/generated/status.rs

src/http/generated/%.rs: build/codegen
	@mkdir -p $(@D)
	build/codegen $(patsubst src/http/generated/%,%,$@) src/http/generated/

build/codegen: src/codegen/main.rs
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTFLAGS) -L build -o $@ --dep-info $@.d $<

build/%: src/%/main.rs $(libhttp)
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTFLAGS) -L build -o $@ --dep-info $@.d $<

$(libhttp): src/http/lib.rs
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTFLAGS) -L build --out-dir build --dep-info $@.d $<

doc/http/index.html: src/http/lib.rs
	$(RUSTDOC) $<

build/tests: src/http/lib.rs
	@mkdir -p $(@D)
	$(RUSTC) $(RUSTFLAGS) --test --dep-info $@.d -o $@ $<

build/quicktests: src/http/lib.rs
	@mkdir -p $(@D)
	$(RUSTC) --test --dep-info $@.d -o $@ $<

# Can't wait for everything to build, optimised too? OK, you can save some time here.
quickcheck: build/quicktests
	build/quicktests --test

check: all build/tests
	build/tests --test

clean:
	rm -rf src/http/generated build
	rm -rf doc/{http/,src/,*.js,*.css}

TAGS:
	ctags -f TAGS --options=$(RUST_CTAGS) -R src

.PHONY: all http examples docs clean check quickcheck
