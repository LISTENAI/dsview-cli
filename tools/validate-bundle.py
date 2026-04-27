#!/usr/bin/env python3

"""Bundle validation helper for DSView CLI releases."""

from __future__ import annotations

import argparse
import json
import os
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


def is_windows_target(target: str) -> bool:
    return "windows" in target


def is_darwin_target(target: str) -> bool:
    return "darwin" in target or "macos" in target


def is_python_dependency(path_or_name: str) -> bool:
    name = Path(path_or_name).name.lower()
    return name == "python" or name.startswith("libpython")


def require_exists(path: Path, label: str) -> None:
    if not path.exists():
        raise FileNotFoundError(f"{label} not found: {path}")


def command_stdout(command: list[str]) -> str:
    try:
        result = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as error:
        raise RuntimeError(f"required validation command not found: {command[0]}") from error
    if result.returncode != 0:
        raise RuntimeError(
            f"validation command failed with exit code {result.returncode}: {' '.join(command)}"
        )
    return result.stdout


def linux_dynamic_values(library: Path) -> dict[str, list[str]]:
    values = {"NEEDED": [], "RPATH": [], "RUNPATH": []}
    output = command_stdout(["readelf", "-d", str(library)])
    for line in output.splitlines():
        for tag in values:
            if f"({tag})" in line and "[" in line and "]" in line:
                values[tag].append(line.split("[", 1)[1].split("]", 1)[0])
    return values


def macos_linked_libraries(library: Path) -> list[str]:
    output = command_stdout(["otool", "-L", str(library)])
    return [
        line.strip().split(" ", 1)[0]
        for line in output.splitlines()[1:]
        if line.strip()
    ]


def macos_rpaths(library: Path) -> list[str]:
    output = command_stdout(["otool", "-l", str(library)])
    rpaths: list[str] = []
    in_rpath_command = False
    for line in output.splitlines():
        stripped = line.strip()
        if stripped == "cmd LC_RPATH":
            in_rpath_command = True
            continue
        if in_rpath_command and stripped.startswith("path "):
            rpaths.append(stripped.split(" ", 2)[1])
            in_rpath_command = False
    return rpaths


def validate_unix_python_runtime(
    bundle_root: Path,
    target: str,
    decode_runtime: Path,
) -> None:
    python_home = bundle_root / "python"
    python_lib = python_home / "lib"
    require_exists(python_home, "Bundled Python runtime directory")
    require_exists(python_lib, "Bundled Python lib directory")

    stdlib_dirs = [
        path for path in python_lib.glob("python*") if path.is_dir()
    ]
    if not any((path / "encodings").is_dir() for path in stdlib_dirs):
        raise FileNotFoundError(
            "Bundled Python stdlib is missing encodings/ under python/lib/pythonX.Y"
        )

    if is_darwin_target(target):
        if not (python_lib / "Python").is_file() and not any(
            python_lib.glob("libpython*.dylib*")
        ):
            raise FileNotFoundError("Bundled macOS Python dynamic library was not found")
        validate_macos_python_links(bundle_root, decode_runtime)
    else:
        if not any(python_lib.glob("libpython*.so*")):
            raise FileNotFoundError("Bundled Linux libpython shared library was not found")
        validate_linux_python_links(bundle_root, decode_runtime)


def validate_linux_python_links(bundle_root: Path, decode_runtime: Path) -> None:
    dynamic_values = linux_dynamic_values(decode_runtime)
    python_dependencies = [
        dependency
        for dependency in dynamic_values["NEEDED"]
        if is_python_dependency(dependency)
    ]
    if not python_dependencies:
        raise RuntimeError("Decode runtime does not declare a libpython dependency")

    python_lib = bundle_root / "python" / "lib"
    for dependency in python_dependencies:
        require_exists(
            python_lib / Path(dependency).name,
            "Bundled Linux libpython dependency",
        )

    runpath = ":".join(dynamic_values["RPATH"] + dynamic_values["RUNPATH"])
    if "$ORIGIN/../python/lib" not in runpath:
        raise RuntimeError(
            "Linux decode runtime RUNPATH must include $ORIGIN/../python/lib"
        )


def validate_macos_python_links(bundle_root: Path, decode_runtime: Path) -> None:
    python_dependencies = [
        dependency
        for dependency in macos_linked_libraries(decode_runtime)
        if is_python_dependency(dependency)
    ]
    if not python_dependencies:
        raise RuntimeError("Decode runtime does not declare a Python dynamic library dependency")

    rpaths = macos_rpaths(decode_runtime)
    if "@loader_path/../python/lib" not in rpaths:
        raise RuntimeError(
            "macOS decode runtime LC_RPATH must include @loader_path/../python/lib"
        )

    python_lib = bundle_root / "python" / "lib"
    for dependency in python_dependencies:
        dependency_name = Path(dependency).name
        require_exists(python_lib / dependency_name, "Bundled macOS Python dependency")
        if dependency.startswith("@loader_path/../python/lib/"):
            continue
        if dependency.startswith("@rpath/"):
            continue
        raise RuntimeError(
            f"macOS Python dependency is not bundle-relative: {dependency}"
        )


def run_smoke_test(
    exe_path: Path, args: list[str], description: str
) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env.pop("PYTHONHOME", None)
    env.pop("PYTHONPATH", None)
    result = subprocess.run(
        [str(exe_path), *args],
        check=False,
        capture_output=True,
        text=True,
        env=env,
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

        exe_name = "dsview-cli.exe" if is_windows_target(args.target) else "dsview-cli"
        exe_path = bundle_root / exe_name
        require_exists(exe_path, "Executable")
        if is_windows_target(args.target):
            for dependency in expected_windows_runtime_dependencies():
                require_exists(bundle_root / dependency, "Windows runtime dependency")
            if not any(bundle_root.glob("python*.dll")):
                raise FileNotFoundError("Bundled Windows Python runtime DLLs were not found")
            python_home = bundle_root / "python"
            if not python_home.is_dir():
                raise FileNotFoundError("python/ directory not found")
            if not (python_home / "Lib").is_dir() and not any(python_home.glob("python*.zip")):
                raise FileNotFoundError(
                    "Bundled Windows Python runtime is missing both Lib/ and pythonXY.zip"
                )

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
        if not is_windows_target(args.target):
            validate_unix_python_runtime(
                bundle_root,
                args.target,
                decode_runtime_dir / decode_runtime_library_name(args.target),
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
