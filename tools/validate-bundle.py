#!/usr/bin/env python3

"""Bundle validation helper for DSView CLI releases."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate the structure and smoke tests for a DSView CLI bundle."
    )
    parser.add_argument(
        "--archive", required=True, type=Path, help="Path to bundle archive (.tar.gz)"
    )
    parser.add_argument("--target", required=True, help="Rust target triple")
    return parser.parse_args()


def runtime_library_name(target: str) -> str:
    if "windows" in target:
        return "dsview_runtime.dll"
    if "darwin" in target or "macos" in target:
        return "libdsview_runtime.dylib"
    return "libdsview_runtime.so"


def decode_runtime_library_name(target: str) -> str:
    if "windows" in target:
        return "dsview_decode_runtime.dll"
    if "darwin" in target or "macos" in target:
        return "libdsview_decode_runtime.dylib"
    return "libdsview_decode_runtime.so"


def expected_windows_runtime_dependencies() -> list[str]:
    return [
        "glib-2.0-0.dll",
        "libusb-1.0.dll",
        "iconv-2.dll",
        "intl-8.dll",
        "pcre2-8.dll",
    ]


def require_exists(path: Path, label: str) -> None:
    if not path.exists():
        raise FileNotFoundError(f"{label} not found: {path}")


def run_smoke_test(
    exe_path: Path, args: list[str], description: str
) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        [str(exe_path), *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        if result.stdout:
            print(result.stdout, file=sys.stderr, end="")
        if result.stderr:
            print(result.stderr, file=sys.stderr, end="")
        raise RuntimeError(
            f"{description} failed with exit code: {result.returncode}"
        )
    print(f"OK {description}")
    return result


def main() -> int:
    args = parse_args()
    require_exists(args.archive, "Archive")

    temp_dir = Path(tempfile.mkdtemp(prefix="dsview-validate-"))
    try:
        with tarfile.open(args.archive, "r:gz") as archive:
            extract_archive(archive, temp_dir)

        entries = list(temp_dir.iterdir())
        if len(entries) != 1:
            raise RuntimeError(
                f"Expected single root directory, found {len(entries)} entries"
            )

        bundle_root = entries[0]
        if not bundle_root.is_dir():
            raise RuntimeError("Archive root is not a directory")

        exe_name = "dsview-cli.exe" if "windows" in args.target else "dsview-cli"
        exe_path = bundle_root / exe_name
        require_exists(exe_path, "Executable")
        if "windows" in args.target:
            for dependency in expected_windows_runtime_dependencies():
                require_exists(bundle_root / dependency, "Windows runtime dependency")

        runtime_dir = bundle_root / "runtime"
        if not runtime_dir.is_dir():
            raise FileNotFoundError("runtime/ directory not found")
        require_exists(runtime_dir / runtime_library_name(args.target), "Runtime library")

        decode_runtime_dir = bundle_root / "decode-runtime"
        if not decode_runtime_dir.is_dir():
            raise FileNotFoundError("decode-runtime/ directory not found")
        require_exists(
            decode_runtime_dir / decode_runtime_library_name(args.target),
            "Decode runtime library",
        )

        decoders_dir = bundle_root / "decoders"
        if not decoders_dir.is_dir():
            raise FileNotFoundError("decoders/ directory not found")
        if not any(decoders_dir.iterdir()):
            raise FileNotFoundError("decoders/ directory is empty")

        resources_dir = bundle_root / "resources"
        if not resources_dir.is_dir():
            raise FileNotFoundError("resources/ directory not found")

        run_smoke_test(exe_path, ["--help"], "dsview-cli --help")
        run_smoke_test(exe_path, ["devices", "list", "--help"], "dsview-cli devices list --help")
        decode_list = run_smoke_test(
            exe_path,
            ["decode", "list", "--format", "json"],
            "dsview-cli decode list --format json",
        )
        decode_payload = json.loads(decode_list.stdout)
        if not isinstance(decode_payload, dict) or not decode_payload.get("decoders"):
            raise RuntimeError("decode list returned an empty decoder registry")
    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)

    print("Bundle validation passed")
    return 0


def extract_archive(archive: tarfile.TarFile, destination: Path) -> None:
    if sys.version_info >= (3, 12):
        archive.extractall(destination, filter="fully_trusted")
    else:
        archive.extractall(destination)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # pragma: no cover - CLI error path
        print(f"Validation failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
