#!/usr/bin/env python3
"""
Regenerate RenderDoc in-app API bindings for `renderdog-sys`.

This script:
1) runs `cargo build -p renderdog-sys --features bindgen` with RENDERDOG_SYS_REGEN_BINDINGS=1
2) parses Cargo JSON messages to locate the build script out_dir
3) copies the generated OUT_DIR/bindings.rs into crates/renderdog-sys/src/bindings_pregenerated.rs

Requirements:
- Python 3.9+
- Rust toolchain + `cargo`
- bindgen prerequisites (libclang) available on the machine

Note:
- docs.rs builds use the pregenerated bindings (no bindgen).
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


def run(cmd: list[str], *, env: dict[str, str] | None = None, cwd: Path | None = None) -> str:
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"command failed ({proc.returncode}): {' '.join(cmd)}\n{proc.stdout}")
    return proc.stdout


def find_out_dir(cargo_json_output: str, package_name: str) -> Path:
    out_dir: Path | None = None
    for line in cargo_json_output.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue

        if msg.get("reason") != "build-script-executed":
            continue

        pkg_id = msg.get("package_id") or ""
        # Cargo's `package_id` string is not stable across formats.
        # On Windows for path dependencies it can look like:
        #   "path+file:///.../renderdog-sys#0.2.0"
        # so we do a simple substring match.
        if package_name not in pkg_id:
            continue

        raw_out_dir = msg.get("out_dir")
        if raw_out_dir:
            out_dir = Path(raw_out_dir)

    if out_dir is None:
        raise RuntimeError(
            f"failed to locate build-script out_dir for package `{package_name}`. "
            "Try re-running with `--verbose`."
        )
    return out_dir


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workspace", default=".", help="Path to the workspace root (default: .)")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Do not write files; only check whether pregenerated bindings are up-to-date.",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print cargo output (useful for diagnosing missing libclang/bindgen issues).",
    )
    args = parser.parse_args()

    root = Path(args.workspace).resolve()
    pregenerated = root / "crates" / "renderdog-sys" / "src" / "bindings_pregenerated.rs"
    if not pregenerated.is_file():
        raise RuntimeError(f"pregenerated bindings not found: {pregenerated}")

    env = dict(os.environ)
    env["RENDERDOG_SYS_REGEN_BINDINGS"] = "1"
    env["RENDERDOG_SYS_VERBOSE"] = "1"

    cargo_cmd = [
        "cargo",
        "build",
        "-p",
        "renderdog-sys",
        "--features",
        "bindgen",
        "--message-format",
        "json",
    ]
    output = run(cargo_cmd, env=env, cwd=root)
    if args.verbose:
        sys.stdout.write(output)

    out_dir = find_out_dir(output, "renderdog-sys")
    generated = out_dir / "bindings.rs"
    if not generated.is_file():
        raise RuntimeError(f"generated bindings not found: {generated}")

    if args.check:
        same = pregenerated.read_bytes() == generated.read_bytes()
        if not same:
            print("bindings are OUT OF DATE (run without --check to update).")
            return 1
        print("bindings are up-to-date.")
        return 0

    tmp = pregenerated.with_suffix(".rs.tmp")
    shutil.copyfile(generated, tmp)
    tmp.replace(pregenerated)
    print(f"updated: {pregenerated}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
