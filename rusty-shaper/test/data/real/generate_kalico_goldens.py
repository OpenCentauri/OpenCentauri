#!/usr/bin/env python3
"""Generate Kalico reference goldens for rusty-shaper regression tests.

Run from the repo root:

    cd /home/paul/carbon/OpenCentauri/rusty-shaper
    python3 test/data/real/generate_kalico_goldens.py

Requires a Kalico checkout at ~/carbon/kalico and `uv`.
"""

import gzip
import json
import os
import re
import subprocess
import tempfile
from pathlib import Path

KALICO_DIR = Path.home() / "carbon" / "kalico"
DATA_DIR = Path(__file__).parent.resolve()

CAPTURES = [
    ("krishlulla", "x", "raw_data_x_lis2dw_20260628_050622.csv"),
    ("krishlulla", "y", "raw_data_y_lis2dw_20260628_050734.csv"),
    ("peterb0288", "x", "raw_data_x_lis2dw_20260628_223144.csv"),
    ("peterb0288", "y", "raw_data_y_lis2dw_20260628_223328.csv"),
    ("atomique13", "x", "raw_data_x_lis2dw_20260619_232243.csv"),
    ("atomique13", "y", "raw_data_y_lis2dw_20260619_232401.csv"),
    ("jaimbo", "x", "raw_data_x_lis2dw_20260629_125021.csv"),
    ("jaimbo", "y", "raw_data_y_lis2dw_20260629_124904.csv"),
    ("harrym", "x", "raw_data_x_lis2dw_20260629_170415.csv"),
    ("harrym", "y", "raw_data_y_lis2dw_20260629_173511.csv"),
    ("lizard_0619", "x", "raw_data_x_lis2dw_20260619_223752.csv"),
    ("lizard_0619", "y", "raw_data_y_lis2dw_20260619_223839.csv"),
    ("lizard_0629", "x", "raw_data_x_lis2dw_20260629_174255.csv"),
    ("lizard_0629", "y", "raw_data_y_lis2dw_20260629_174451.csv"),
]

SHAPER_RE = re.compile(
    r"Fitted shaper '([a-z0-9_]+)' frequency = ([0-9.]+) Hz "
    r"\(vibrations = ([0-9.]+)%, smoothing ~= ([0-9.]+)\)"
)
MAX_ACCEL_RE = re.compile(
    r"To avoid too much smoothing with '([a-z0-9_]+)', suggested "
    r"max_accel <= ([0-9]+) mm/sec\^2"
)
RECOMMENDED_RE = re.compile(
    r"Recommended shaper is ([a-z0-9_]+) @ ([0-9.]+) Hz"
)


def run_kalico(csv_path: Path) -> dict:
    env = os.environ.copy()
    env["MPLBACKEND"] = "Agg"
    out_png = f"/tmp/kalico_{csv_path.stem}.png"
    cmd = [
        "uv", "run", "python",
        str(KALICO_DIR / "scripts" / "calibrate_shaper.py"),
        "--output", out_png,
        str(csv_path),
    ]
    result = subprocess.run(
        cmd,
        cwd=KALICO_DIR,
        env=env,
        capture_output=True,
        text=True,
        check=True,
    )
    return parse_output(result.stdout)


def parse_output(stdout: str) -> dict:
    shapers = {}
    for match in SHAPER_RE.finditer(stdout):
        name = match.group(1)
        shapers[name] = {
            "name": name,
            "freq": float(match.group(2)),
            "vibrs": float(match.group(3)) / 100.0,
            "smoothing": float(match.group(4)),
        }
    for match in MAX_ACCEL_RE.finditer(stdout):
        name = match.group(1)
        shapers[name]["max_accel"] = float(match.group(2))

    rec_match = RECOMMENDED_RE.search(stdout)
    if rec_match is None:
        raise RuntimeError("Kalico did not produce a recommendation")

    return {
        "recommended": {
            "name": rec_match.group(1),
            "freq": float(rec_match.group(2)),
        },
        "all_shapers": list(shapers.values()),
    }


def main() -> None:
    kalico_rev = subprocess.run(
        ["git", "rev-parse", "--short", "HEAD"],
        cwd=KALICO_DIR,
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()

    for author, axis, csv_name in CAPTURES:
        compressed_path = DATA_DIR / f"{csv_name}.gz"
        with tempfile.NamedTemporaryFile(suffix=".csv") as raw_file:
            with gzip.open(compressed_path, "rb") as compressed_file:
                raw_file.write(compressed_file.read())
            raw_file.flush()
            golden = run_kalico(Path(raw_file.name))
        golden["source"] = "kalico"
        golden["kalico_rev"] = kalico_rev
        golden["input_csv"] = csv_name

        out_path = DATA_DIR / f"{author}_{axis}.json"
        with open(out_path, "w") as f:
            json.dump(golden, f, indent=2)
            f.write("\n")
        print(f"{author}_{axis}: {golden['recommended']['name']} @ "
              f"{golden['recommended']['freq']:.1f} Hz → {out_path.name}")


if __name__ == "__main__":
    main()
