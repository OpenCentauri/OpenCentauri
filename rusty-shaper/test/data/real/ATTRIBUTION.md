# Real-World Test Data Attribution

Fourteen raw accelerometer CSVs from Centauri Carbon owners, posted to the
OpenCentauri Discord for input-shaper calibration work. They are checked
into the repo so the regression test and benchmark exercise against
real-world data instead of synthetic fixtures. The repository stores the
captures as deterministic `.csv.gz` files in Git LFS; they are decompressed
only by the test and benchmark fixtures.

The golden JSON files (`*_x.json`, `*_y.json`) are generated from Kalico's
`scripts/calibrate_shaper.py`, not from `rusty-shaper`. The regression test
asserts that `rusty-shaper` matches Kalico within explicit tolerances. See
`generate_kalico_goldens.py` for the driver.

When adding a capture, add a section below with:

- Discord handle + user ID
- Discord message URL the CSV was attached to
- Test setup (Cosmo version, sensor type, toolhead)
- Approximate printer / input-shaper config (`[input_shaper]` block,
  shaper frequency range, relevant `machine.cfg` includes if known)
- Capture time range and approximate sampling rate

Config details are included when the user provided them in Discord. If a
field is blank, it wasn't shared — please don't guess.

## Captures

### `krishlulla` — `#input-shaper-testing`, 2026-06-28

- **Author**: `krishlulla` (Discord ID `1258897253163339816`)
- **Message URL**: https://discord.com/channels/1367538416539013122/1520618527554535484/1520660973583859743
- **Files**: `raw_data_x_lis2dw_20260628_050622.csv.gz`, `raw_data_y_lis2dw_20260628_050734.csv.gz`
- **Sensor**: LIS2DW
- **Config**: Not shared in the thread.
- **Capture time**: ~70 s per axis (~111 k samples each, ~5.5 MB)
- **Note**: Original dataset used for the v0.1 Kalico-parity validation.
- **Recommendation**: ZV on both axes (51.6 Hz X, 45.2 Hz Y).

### `peterb0288` — `#input-shaper-testing`, 2026-06-28

- **Author**: `peterb0288` (Discord ID `1451877684572327936`)
- **Message URL**: https://discord.com/channels/1367538416539013122/1520618527554535484/1520923298924335186
- **Message body**: "Here you are, Cosmo 0.7"
- **Files**: `raw_data_x_lis2dw_20260628_223144.csv.gz`, `raw_data_y_lis2dw_20260628_223328.csv.gz`
- **Sensor**: LIS2DW
- **Firmware**: Cosmo 0.7
- **Config**: Not shared in the thread.
- **Capture time**: ~70 s per axis (~111 k samples each, ~5.1–5.2 MB)
- **Note**: Different recommended shaper (MZV on both axes) and a
  ~5 Hz higher frequency band than `krishlulla`, so the regression test
  exercises both ZV- and MZV-winning cases.
- **Recommendation**: MZV on both axes (56.6 Hz X, 46.8 Hz Y).

### `atomique13` — `#input-shaper-testing`, 2026-06-19

- **Author**: `atomique13` (Discord ID `297049830885294082`)
- **Files**: `raw_data_x_lis2dw_20260619_232243.csv.gz`, `raw_data_y_lis2dw_20260619_232401.csv.gz`
- **Sensor**: LIS2DW
- **Config**: Not shared in the thread.
- **Capture time**: ~45 s per axis (~71 k samples each, ~3.4 MB — smaller
  than the other two captures)
- **Note**: Mixed-axis recommendations (MZV on X, ZV on Y), exercises both
  branches of the per-shaper selection logic in a single capture.
- **Recommendation**: MZV on X (56.2 Hz), ZV on Y (46.8 Hz).

### `jaimbo` — `#input-shaper-testing`, 2026-06-29

- **Author**: `jaimbo` (Discord ID `181036720353968128`)
- **Message URL**: https://discord.com/channels/1367538416539013122/1520618527554535484/1521136078936080434
- **Message body**: "- Proxima Toolhead\n- Constellation HF\n- COSMOS load-cell-driver branch\n- Constant OOM ✅"
- **Files**: `raw_data_x_lis2dw_20260629_125021.csv.gz`, `raw_data_y_lis2dw_20260629_124904.csv.gz`
- **Sensor**: LIS2DW
- **Toolhead**: Proxima Toolhead + Constellation HF
- **Firmware**: COSMOS load-cell-driver branch
- **Config**: Not shared in the thread.
- **Capture time**: ~45 s per axis (~71 k samples each, ~3.4 MB)
- **Note**: Mixed-axis recommendations (MZV on X, ZV on Y) on a non-stock
  toolhead; highest-frequency X peak in the current dataset (59.4 Hz).
- **Recommendation**: MZV on X (59.4 Hz), ZV on Y (48.4 Hz).

### `harrym` — `#input-shaper-testing`, 2026-06-29

- **Author**: `harrym_84915` (Discord ID `1313439410213224458`)
- **Message URLs**: 
  - X: https://discord.com/channels/1367538416539013122/1520618527554535484/1521203794451628083
  - Y: https://discord.com/channels/1367538416539013122/1520618527554535484/1521208220041875486
- **Files**: `raw_data_x_lis2dw_20260629_170415.csv.gz`, `raw_data_y_lis2dw_20260629_173511.csv.gz`
- **Sensor**: LIS2DW
- **Toolhead**: TZ4 hotend + Constellation + "magic dancer" toolhead
- **Config**: Not shared in the thread.
- **Capture time**: ~45 s per axis (~72 k samples each, ~3.4 MB)
- **Note**: First Y capture was pulled too early and only 8 KB; this is the
  second, successful Y capture. ZV wins on both axes.
- **Recommendation**: ZV on both axes (56.6 Hz X, 47.0 Hz Y).

### `lizard_0619` — `#input-shaper-testing`, 2026-06-19

- **Author**: `clogged_nozzl3` (Discord ID `336097089715175426`)
- **Message URL**: https://discord.com/channels/1367538416539013122/1520618527554535484/1521213546673733834
- **Files**: `raw_data_x_lis2dw_20260619_223752.csv.gz`, `raw_data_y_lis2dw_20260619_223839.csv.gz`
- **Sensor**: LIS2DW
- **Config**: Not shared in the thread.
- **Capture time**: ~45 s per axis (~72 k samples each, ~3.5 MB)
- **Note**: First of two captures from the same printer (see `lizard_0629`
  below). Useful longitudinal check: X peak shifted ~0.4 Hz lower and Y
  peak shifted ~0.8 Hz lower between the two dates.
- **Recommendation**: ZV on both axes (61.4 Hz X, 52.2 Hz Y).

### `lizard_0629` — `#input-shaper-testing`, 2026-06-29

- **Author**: `clogged_nozzl3` (Discord ID `336097089715175426`)
- **Message URL**: https://discord.com/channels/1367538416539013122/1520618527554535484/1521213546673733834
- **Files**: `raw_data_x_lis2dw_20260629_174255.csv.gz`, `raw_data_y_lis2dw_20260629_174451.csv.gz`
- **Sensor**: LIS2DW
- **Config**: Not shared in the thread.
- **Capture time**: ~45 s per axis (~72 k samples each, ~3.5 MB)
- **Note**: Second of two captures from the same printer, taken when
  re-running the test "just now to be safe" after an earlier ShakeTune
  Docker run on 2026-06-19.
- **Recommendation**: ZV on both axes (61.0 Hz X, 51.4 Hz Y).

## How the golden outputs were generated

The JSON goldens are produced by Kalico's `scripts/calibrate_shaper.py` using
the helper in this directory:

```bash
cd /home/paul/carbon/OpenCentauri/rusty-shaper
python3 test/data/real/generate_kalico_goldens.py
```

This requires a Kalico checkout at `~/carbon/kalico` and `uv`. The script
runs `calibrate_shaper.py --output /tmp/kalico_<name>.png <csv>` for each
capture, parses the stdout, and writes `{author}_{axis}.json` with the
recommended shaper and per-shaper metrics.

If you are generating goldens from a capture you just took, please also
paste the relevant `[input_shaper]` block and any non-default shaper
frequency range from your `printer.cfg` into the attribution section above.
That lets future readers understand what config produced the data.

To regenerate goldens after an intentional calibration change in Kalico or
`rusty-shaper`, re-run the script and update the regression-test tolerances
if the new results are still within the Kalico-parity budget (< 0.5 Hz freq
drift).

## If you find a problem with attribution

If any of these files should not be in the repo (consent issue, wrong
data, etc.), open an issue or ping `@pdscomp` and we'll remove or replace
them. Do not post these files outside the OpenCentauri project without
checking with the original author.
