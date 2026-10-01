#!/bin/bash
set -euo pipefail

target="${TARGET:-armv7-unknown-linux-musleabihf}"

# Build both the CLI binary and the cdylib (.so) for Kalico ctypes integration.
cross build --locked --release --target "$target"

# Locate the cross output directory and copy both artefacts to a flat
# dist/ directory so callers have a single well-known location.
dist="dist/${target}"
mkdir -p "$dist"

out="target/${target}/release"

cp "${out}/rusty-shaper"            "${dist}/rusty-shaper"
cp "${out}/librusty_shaper.so"      "${dist}/librusty_shaper.so"

echo "Artefacts written to ${dist}/:"
ls -lh "${dist}/"
