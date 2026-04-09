#!/usr/bin/env python3

"""Bundle packaging helper for DSView CLI releases."""

from __future__ import annotations

import argparse
import sys
import tarfile
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a versioned DSView CLI release bundle."
    )
    parser.add_argument("--exe", required=True, type=Path, help="Path to built executable")
    parser.add_argument(
        "--runtime", required=True, type=Path, help="Path to built runtime library"
    )
    parser.add_argument(
        "--resources", required=True, type=Path, help="Path to DSView resource directory"
    )
    parser.add_argument(
        "--output", required=True, type=Path, help="Path to output archive (.tar.gz)"
    )
    parser.add_argument("--version", required=True, help="Version string")
    parser.add_argument("--target", required=True, help="Rust target triple")
    return parser.parse_args()


def ensure_exists(path: Path, label: str) -> None:
    if not path.exists():
        raise FileNotFoundError(f"{label} not found: {path}")


def add_file(archive: tarfile.TarFile, source: Path, destination: str) -> None:
    archive.add(source, arcname=destination, recursive=False)


def main() -> int:
    args = parse_args()

    ensure_exists(args.exe, "Executable")
    ensure_exists(args.runtime, "Runtime library")
    ensure_exists(args.resources, "Resources directory")

    archive_root = f"dsview-cli-{args.version}-{args.target}"
    exe_name = "dsview-cli.exe" if "windows" in args.target else "dsview-cli"

    required_resources = [
        "DSLogicPlus.fw",
        "DSLogic.fw",  # fallback resource kept for compatibility
        "DSLogicPlus.bin",
        "DSLogicPlus-pgl12.bin",
    ]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(args.output, "w:gz") as archive:
        add_file(archive, args.exe, f"{archive_root}/{exe_name}")
        add_file(archive, args.runtime, f"{archive_root}/runtime/{args.runtime.name}")

        for resource_name in required_resources:
            resource_path = args.resources / resource_name
            if resource_path.exists():
                add_file(archive, resource_path, f"{archive_root}/resources/{resource_name}")

    print(f"Bundle created: {args.output}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # pragma: no cover - CLI error path
        print(f"Packaging failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
