#!/bin/sh

# Host verification gate. ARM/cross checks and the benchmark are documented in
# TEST.md because they require a target toolchain or intentionally take longer.

set -eu

cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo build --release --locked
