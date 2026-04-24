#!/usr/bin/env python3

"""Bundle packaging helper for DSView CLI releases."""

from __future__ import annotations

import argparse
import os
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
        "--decode-runtime",
        required=True,
        type=Path,
        help="Path to built decode runtime library",
    )
    parser.add_argument(
        "--resources", required=True, type=Path, help="Path to DSView resource directory"
    )
    parser.add_argument(
        "--decoder-dir",
        required=True,
        type=Path,
        help="Path to the DSView decoder scripts directory",
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


def ensure_file(path: Path, label: str) -> None:
    ensure_exists(path, label)
    if not path.is_file():
        raise FileNotFoundError(f"{label} is not a file: {path}")


def ensure_directory(path: Path, label: str) -> None:
    ensure_exists(path, label)
    if not path.is_dir():
        raise FileNotFoundError(f"{label} is not a directory: {path}")


def add_file(archive: tarfile.TarFile, source: Path, destination: str) -> None:
    archive.add(source, arcname=destination, recursive=False)


def should_skip_decoder_path(path: Path) -> bool:
    return any(part == "__pycache__" for part in path.parts) or path.suffix in {".pyc", ".pyo"}


def should_skip_python_path(path: Path) -> bool:
    return (
        any(part == "__pycache__" for part in path.parts)
        or path.suffix in {".pyc", ".pyo"}
        or path.parts[:1] == ("Lib",) and len(path.parts) > 1 and path.parts[1] == "site-packages"
    )


def add_directory(archive: tarfile.TarFile, source: Path, destination: str) -> None:
    archive.add(source, arcname=destination, recursive=False)
    for child in sorted(source.rglob("*")):
        if should_skip_decoder_path(child.relative_to(source)):
            continue
        archive.add(child, arcname=f"{destination}/{child.relative_to(source)}", recursive=False)


def add_directory_filtered(
    archive: tarfile.TarFile,
    source: Path,
    destination: str,
    skip_predicate,
) -> None:
    archive.add(source, arcname=destination, recursive=False)
    for child in sorted(source.rglob("*")):
        if skip_predicate(child.relative_to(source)):
            continue
        archive.add(child, arcname=f"{destination}/{child.relative_to(source)}", recursive=False)


def vcpkg_triplet_for_target(target: str) -> str:
    if "windows" not in target:
        raise ValueError(f"target is not a Windows target: {target}")
    if target.startswith("x86_64-"):
        return "x64-windows"
    if target.startswith("aarch64-"):
        return "arm64-windows"
    raise ValueError(f"unsupported Windows target: {target}")


def windows_runtime_dependency_dir(target: str) -> Path:
    vcpkg_root = os.environ.get("VCPKG_ROOT") or os.environ.get("DSVIEW_VCPKG_ROOT")
    if not vcpkg_root:
        raise FileNotFoundError(
            "Windows packaging requires VCPKG_ROOT or DSVIEW_VCPKG_ROOT so dependent DLLs can be bundled"
        )

    triplet = os.environ.get("DSVIEW_VCPKG_TRIPLET") or vcpkg_triplet_for_target(target)
    dependency_dir = Path(vcpkg_root) / "installed" / triplet / "bin"
    ensure_directory(dependency_dir, "Windows runtime dependency directory")
    return dependency_dir


def windows_dependency_dlls(target: str, runtime_name: str) -> list[Path]:
    dependency_dir = windows_runtime_dependency_dir(target)
    dlls = sorted(
        path
        for path in dependency_dir.glob("*.dll")
        if path.name.lower() != runtime_name.lower()
    )
    if not dlls:
        raise FileNotFoundError(
            f"No Windows runtime dependency DLLs were found under {dependency_dir}"
        )
    return dlls


def windows_python_runtime_root() -> Path:
    root = Path(sys.base_exec_prefix)
    ensure_directory(root, "Windows Python runtime root")
    return root


def windows_python_runtime_dlls() -> list[Path]:
    root = windows_python_runtime_root()
    dlls = sorted(
        {
            *root.glob("python*.dll"),
            *root.glob("vcruntime*.dll"),
        }
    )
    if not dlls:
        raise FileNotFoundError(f"No Python runtime DLLs were found under {root}")
    return dlls


def add_windows_python_runtime(archive: tarfile.TarFile, archive_root: str) -> None:
    python_root = windows_python_runtime_root()

    for dll in windows_python_runtime_dlls():
        add_file(archive, dll, f"{archive_root}/{dll.name}")

    lib_dir = python_root / "Lib"
    ensure_directory(lib_dir, "Windows Python Lib directory")
    add_directory_filtered(archive, lib_dir, f"{archive_root}/python/Lib", should_skip_python_path)

    dll_dir = python_root / "DLLs"
    if dll_dir.is_dir():
        add_directory_filtered(archive, dll_dir, f"{archive_root}/python/DLLs", should_skip_python_path)

    stdlib_zip = python_root / f"python{sys.version_info.major}{sys.version_info.minor}.zip"
    if stdlib_zip.is_file():
        add_file(archive, stdlib_zip, f"{archive_root}/python/{stdlib_zip.name}")


def main() -> int:
    args = parse_args()

    ensure_file(args.exe, "Executable")
    ensure_file(args.runtime, "Runtime library")
    ensure_file(args.decode_runtime, "Decode runtime library")
    ensure_directory(args.resources, "Resources directory")
    ensure_directory(args.decoder_dir, "Decoder scripts directory")

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
        add_file(
            archive,
            args.decode_runtime,
            f"{archive_root}/decode-runtime/{args.decode_runtime.name}",
        )
        add_directory(archive, args.decoder_dir, f"{archive_root}/decoders")
        if "windows" in args.target:
            for dependency in windows_dependency_dlls(args.target, args.runtime.name):
                add_file(
                    archive,
                    dependency,
                    f"{archive_root}/{dependency.name}",
                )
            add_windows_python_runtime(archive, archive_root)

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
