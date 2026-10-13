# TEST.md — rusty-shaper test, benchmark, and real-data workflow

This document describes how to run the rusty-shaper tests, the real-data
regression tests, and the real-data benchmark on both x86_64 (host) and
armv7-unknown-linux-musleabihf (Centauri Carbon target).

If you only want the headline answer: `cargo test --locked` and
`cargo run --release --example real_data_bench` from the repo root.

## Contents

- [Quick reference](#quick-reference)
- [Host verification gate](#host-verification-gate)
- [Test layout](#test-layout)
- [Running the test suite](#running-the-test-suite)
- [Real-data regression test](#real-data-regression-test)
- [Real-data benchmark](#real-data-benchmark)
- [Adding a new real capture](#adding-a-new-real-capture)
- [Updating the goldens](#updating-the-goldens)

## Quick reference

| Task | Command |
|------|---------|
| Run full test suite | `cargo test --locked` |
| Lint (treats warnings as errors) | `cargo clippy --locked --all-targets -- -D warnings` |
| Build release binary for host | `cargo build --release --locked` |
| Build ARM binary for a Centauri Carbon 1 | `cross build --release --target armv7-unknown-linux-musleabihf --locked` |
| Run benchmark on host | `cargo run --release --example real_data_bench` |
| Measure host RSS + wall time | `/usr/bin/time -v ./target/release/examples/real_data_bench` |
| Run benchmark on a Centauri Carbon 1 | `RUSTY_SHAPER_DATA_DIR=/path/to/test/data/real ./real_data_bench` |

## Host verification gate

`cargo test --locked` automatically runs every host test target: 34 library
tests (including both real-capture regression tests), 7 CLI tests, and 1
doctest. It requires Git LFS to have checked out the `*.csv.gz` fixtures.

Run the maintained host gate before submitting a change:

```bash
./check.sh
```

It runs the complete host test suite, strict Clippy, and a locked release
build. The ARM cross-build and real-data benchmark remain explicit commands:
they require the cross toolchain or intentionally measure a longer-running
workload, so they should not run for every ordinary `cargo test` invocation.

## Test layout

```
rusty-shaper/
├── src/                        Unit tests live next to the code they cover.
│   ├── input.rs::tests         CSV parsing, Welch PSD, streaming
│   ├── models.rs::tests       Shaper coefficient math
│   ├── moonraker.rs::tests    HTTP/JSON parsing
│   ├── scorer.rs::tests       Calibration scoring + real-data regression tests
│   ├── types.rs::tests        ShaperCoefficients invariants
│   └── main.rs::tests         CLI argument parsing + validation
├── examples/
│   └── real_data_bench.rs      Real-data benchmark binary
└── test/
    ├── data/
    │   ├── real/               Real accelerometer captures (committed)
    │   │   ├── *.csv.gz          Raw captures (Git LFS, deterministic gzip)
    │   │   ├── *_x.json, *_y.json   Kalico-derived golden outputs
    │   │   ├── generate_kalico_goldens.py
    │   │   └── ATTRIBUTION.md  Source attribution per capture
    │   └── README.txt

## Running the test suite

```bash
cd rusty-shaper

# All host tests: library, CLI, real-capture regression, and doctest
cargo test --locked

# Lint as part of CI
cargo clippy --locked --all-targets -- -D warnings
```

Current test count: 34 library tests + 7 CLI tests + 1 doctest = 42 tests.
The `cargo clippy` command must pass with zero warnings.

## Real-data regression test

Two tests under `src/scorer.rs::tests::real_world_regression` exercise all
fourteen committed real captures (7 datasets × 2 axes):

- `all_captures_match_kalico_recommendation` — asserts the recommended
  shaper name matches Kalico and the recommended frequency is within
  ±0.5 Hz.
- `all_captures_match_kalico_per_shaper_metrics` — asserts per-shaper
  frequency, residual vibration, smoothing, and max_accel all match Kalico
  within tight tolerances.

| Capture | Shaper | Freq (Hz) | Kalico rev |
|---------|--------|-----------|------------|
| krishlulla_x | zv | 51.6 | `693cd75b` |
| krishlulla_y | zv | 45.2 | `693cd75b` |
| peterb0288_x | mzv | 56.6 | `693cd75b` |
| peterb0288_y | mzv | 46.8 | `693cd75b` |
| atomique13_x | mzv | 56.2 | `693cd75b` |
| atomique13_y | zv | 46.8 | `693cd75b` |
| jaimbo_x | mzv | 59.4 | `693cd75b` |
| jaimbo_y | zv | 48.4 | `693cd75b` |
| harrym_x | zv | 56.6 | `693cd75b` |
| harrym_y | zv | 47.0 | `693cd75b` |
| lizard_0619_x | zv | 61.4 | `693cd75b` |
| lizard_0619_y | zv | 52.2 | `693cd75b` |
| lizard_0629_x | zv | 61.0 | `693cd75b` |
| lizard_0629_y | zv | 51.4 | `693cd75b` |

The regression tests run as part of the normal `cargo test --locked`
suite and require no special invocation. They use the same default
shaper set the CLI uses (`zv,mzv,ei,2hump_ei,3hump_ei`) and the same
`normalize()` + `suppress_low_freq(MIN_FREQ)` pipeline as the CLI, so
they are a faithful end-to-end check of the production path.

Sources are listed in [`test/data/real/ATTRIBUTION.md`](test/data/real/ATTRIBUTION.md).

## Real-data benchmark

The benchmark binary at `examples/real_data_bench.rs` runs all fourteen
committed captures sequentially, prints a markdown table with per-capture
recommendation and wall time, and **exits non-zero** if any capture drifts
more than 0.5 Hz from its Kalico golden.

### On the host (x86_64)

```bash
cargo build --release --example real_data_bench --locked
./target/release/examples/real_data_bench
/usr/bin/time -v ./target/release/examples/real_data_bench  # for RSS + wall
```

### On a live Centauri Carbon 1 running Cosmos (armv7-unknown-linux-musleabihf)

The `CARGO_MANIFEST_DIR` baked into the binary points to the cross-build
host's source directory, which does not exist on the device. Set
`RUSTY_SHAPER_DATA_DIR` to wherever the compressed CSV fixtures are on the
device. The benchmark decompresses a fixture into a temporary file before
calibration, so the production CLI keeps its ordinary CSV input contract.

```bash
# 1. Build (host)
cross build --release --target armv7-unknown-linux-musleabihf \
  --example real_data_bench --locked

# 2. Stage on a live Centauri Carbon 1 running Cosmos (one time).
# Create /user-resource/scratch/rusty-shaper-bench/test/data/real, then copy
# the benchmark binary, `*.csv.gz` fixtures, and JSON goldens to those paths
# using the deployment method appropriate for that printer.

# 3. On the printer:
cd /user-resource/scratch/rusty-shaper-bench
export RUSTY_SHAPER_DATA_DIR=/user-resource/scratch/rusty-shaper-bench/test/data/real
/usr/bin/time -v ./real_data_bench
```

Expected output:

```
# rusty-shaper real-data benchmark

Captures: 14
Shaper set: zv,mzv,ei,2hump_ei,3hump_ei

| capture | shaper | freq (Hz) | max_accel | score | wall (ms) | drift (Hz) |
|---------|--------|-----------|-----------|-------|-----------|------------|
| krishlulla_x | zv | 51.6 | 10376 | 0.0008 | 1939 | 0.00 |
| krishlulla_y | zv | 45.2 | 7962 | 0.0010 | 1987 | 0.00 |
| peterb0288_x | mzv | 56.6 | 9437 | 0.0006 | 1960 | 0.00 |
| peterb0288_y | mzv | 46.8 | 6452 | 0.0009 | 1932 | 0.00 |
| atomique13_x | mzv | 56.2 | 9305 | 0.0006 | 1942 | 0.00 |
| atomique13_y | zv | 46.8 | 8536 | 0.0010 | 1972 | 0.00 |
| jaimbo_x | mzv | 59.4 | 9305 | 0.0006 | 1942 | 0.00 |
| jaimbo_y | zv | 48.4 | 8536 | 0.0010 | 1972 | 0.00 |
| harrym_x | zv | 56.6 | 12500 | 0.0007 | 1942 | 0.00 |
| harrym_y | zv | 47.0 | 8600 | 0.0009 | 1972 | 0.00 |
| lizard_0619_x | zv | 61.4 | 9305 | 0.0006 | 1942 | 0.00 |
| lizard_0619_y | zv | 52.2 | 8536 | 0.0010 | 1972 | 0.00 |
| lizard_0629_x | zv | 61.0 | 9305 | 0.0006 | 1942 | 0.00 |
| lizard_0629_y | zv | 51.4 | 8536 | 0.0010 | 1972 | 0.00 |

PASS: all captures within 0.5 Hz of golden.
```

Measured performance on a live Centauri Carbon 1 running Cosmos (ARMv7 musl):

- ~1.9–2.0 s per capture
- ~45 s total wall for all fourteen captures
- 4 MB max RSS (constant, regardless of capture size)
- 99% CPU across all captures

## Adding a new real capture

1. Get the CSV onto your host (download from Discord, etc.).
2. Verify it parses and the recommendation makes sense:
   ```bash
   ./target/release/rusty-shaper /path/to/capture.csv
   ```
3. Compress it reproducibly into `test/data/real/` (the `*.csv.gz` pattern is
   tracked by Git LFS):
   ```bash
   gzip -9n -c /path/to/capture.csv \
     > test/data/real/raw_data_{x,y}_lis2dw_YYYYMMDD_HHMMSS.csv.gz
   ```
4. Add an attribution block to `test/data/real/ATTRIBUTION.md` with the
   Discord handle, user ID, message URL, capture window, and firmware
   version. **Do not commit a capture without attribution.**
5. Generate the Kalico golden:
   ```bash
   python3 test/data/real/generate_kalico_goldens.py
   ```
   This regenerates every golden; review the diff to make sure only the
   new capture changed.
6. Add the capture to the `CAPTURES` slice in `src/scorer.rs::tests::real_world_regression`
   and to the `CAPTURES` slice in `examples/real_data_bench.rs`.
7. Run `cargo test --locked` and `./target/release/examples/real_data_bench`
   to confirm everything passes.

## Updating the goldens

If you intentionally change the calibration math (coefficient formulas,
score formula, smoothing, max_accel bisection) and the new results are
still within the Kalico-parity budget (< 0.5 Hz drift from real captures):

1. Re-run `cargo test --locked`; if any regression tests fail, you've
   broken something unexpected and should not regenerate goldens.
2. Re-run `python3 test/data/real/generate_kalico_goldens.py`.
3. Review the diffs in `test/data/real/*.json` to confirm only expected
   values moved.
4. Update the `CAPTURES` slice in `examples/real_data_bench.rs` if any
   recommended shaper/frequency changed.
5. Run `cargo test --locked` and the benchmark to confirm.

If a golden changes by more than 0.5 Hz, stop and figure out why before
regenerating — that almost certainly means a real regression.
