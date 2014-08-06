#!/bin/sh

# Pre-build code generation script.

codegen='codegen/target/codegen'
generated_src_dir='src/http/generated/'

cargo build --manifest-path codegen/Cargo.toml
mkdir -p ${generated_src_dir}
${codegen} read_method.rs ${generated_src_dir}
${codegen} status.rs ${generated_src_dir}
