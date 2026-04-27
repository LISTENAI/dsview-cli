#!/usr/bin/env python3

"""Bundle packaging helper for DSView CLI releases."""

from __future__ import annotations

import argparse
import os
import posixpath
import shutil
import subprocess
import sys
import sysconfig
import tarfile
import tempfile
from pathlib import Path, PurePosixPath


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


def add_regular_file(archive: tarfile.TarFile, source: Path, destination: str) -> None:
    resolved = source.resolve()
    stat = resolved.stat()
    info = tarfile.TarInfo(destination)
    info.size = stat.st_size
    info.mode = stat.st_mode & 0o777
    info.mtime = stat.st_mtime
    with resolved.open("rb") as file:
        archive.addfile(info, file)


def should_skip_decoder_path(path: Path) -> bool:
    return any(part == "__pycache__" for part in path.parts) or path.suffix in {".pyc", ".pyo"}


def should_skip_python_path(path: Path) -> bool:
    skipped_directory_names = {
        "__pycache__",
        "site-packages",
        "dist-packages",
        "test",
        "tests",
        "ensurepip",
        "idlelib",
        "tkinter",
        "turtledemo",
    }
    return (
        any(
            part in skipped_directory_names or part.startswith("config-")
            for part in path.parts
        )
        or path.suffix in {".a", ".pyc", ".pyo"}
        or path.name.startswith("_test")
        or path.name.startswith("_tkinter.")
        or path.name.startswith("_ctypes_test.")
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


def copy_directory_filtered(
    source: Path,
    destination: Path,
    skip_predicate,
) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for child in sorted(source.rglob("*")):
        relative_path = child.relative_to(source)
        if skip_predicate(relative_path):
            continue

        copied_path = destination / relative_path
        if child.is_symlink():
            resolved = child.resolve()
            if not resolved.is_file():
                continue
            copied_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(resolved, copied_path)
        elif child.is_dir():
            copied_path.mkdir(parents=True, exist_ok=True)
        elif child.is_file():
            copied_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(child, copied_path)


def should_never_skip(_: Path) -> bool:
    return False


def vcpkg_triplet_for_target(target: str) -> str:
    if "windows" not in target:
        raise ValueError(f"target is not a Windows target: {target}")
    if target.startswith("x86_64-"):
        return "x64-windows"
    if target.startswith("aarch64-"):
        return "arm64-windows"
    raise ValueError(f"unsupported Windows target: {target}")


def is_windows_target(target: str) -> bool:
    return "windows" in target


def is_darwin_target(target: str) -> bool:
    return "darwin" in target or "macos" in target


def python_version_dir() -> str:
    return f"python{sys.version_info.major}.{sys.version_info.minor}"


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


def is_python_dependency(path_or_name: str) -> bool:
    name = Path(path_or_name).name.lower()
    return name == "python" or name.startswith("libpython")


def command_stdout(command: list[str]) -> str:
    try:
        result = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError:
        return ""
    if result.returncode != 0:
        return ""
    return result.stdout


def checked_command_stdout(command: list[str]) -> str:
    try:
        result = subprocess.run(
            command,
            check=True,
            capture_output=True,
            text=True,
        )
    except FileNotFoundError as error:
        raise FileNotFoundError(f"required command not found: {command[0]}") from error
    return result.stdout


def require_tool(name: str) -> str:
    tool = shutil.which(name)
    if not tool:
        raise FileNotFoundError(f"{name} is required for macOS Python runtime bundling")
    return tool


def linux_linked_python_dependencies(library: Path) -> list[tuple[str, str]]:
    dependencies: list[tuple[str, str]] = []
    output = command_stdout(["readelf", "-d", str(library)])
    for line in output.splitlines():
        if "Shared library:" not in line or "[" not in line or "]" not in line:
            continue
        dependency = line.split("[", 1)[1].split("]", 1)[0]
        if is_python_dependency(dependency):
            dependencies.append((dependency, Path(dependency).name))
    return dependencies


def macos_linked_python_dependencies(library: Path) -> list[tuple[str, str]]:
    dependencies: list[tuple[str, str]] = []
    output = command_stdout(["otool", "-L", str(library)])
    for line in output.splitlines()[1:]:
        dependency = line.strip().split(" ", 1)[0]
        if is_python_dependency(dependency):
            dependencies.append((dependency, Path(dependency).name))
    return dependencies


def macos_linked_libraries(library: Path) -> list[str]:
    output = checked_command_stdout(["otool", "-L", str(library)])
    return [
        line.strip().split(" ", 1)[0]
        for line in output.splitlines()[1:]
        if line.strip()
    ]


def linked_python_dependencies(target: str, library: Path) -> list[tuple[str, str]]:
    if is_darwin_target(target):
        return macos_linked_python_dependencies(library)
    if is_windows_target(target):
        return []
    return linux_linked_python_dependencies(library)


def macos_python_archive_path(dependency_or_source: str, fallback_name: str) -> str:
    parts = Path(dependency_or_source).parts
    for index, part in enumerate(parts):
        if part.endswith(".framework"):
            return Path(*parts[index:]).as_posix()
    return f"lib/{fallback_name}"


def macos_framework_version_root(path: Path) -> Path | None:
    for index, part in enumerate(path.parts):
        if not part.endswith(".framework"):
            continue

        framework_root = Path(*path.parts[: index + 1])
        if index + 2 < len(path.parts) and path.parts[index + 1] == "Versions":
            return Path(*path.parts[: index + 3])

        framework_version = sysconfig.get_config_var("VERSION") or python_version_dir().removeprefix("python")
        version_root = framework_root / "Versions" / framework_version
        return version_root if version_root.is_dir() else None

    return None


def macos_framework_archive_version_root(version_root: Path) -> PurePosixPath:
    framework_root = version_root.parent.parent
    return PurePosixPath(framework_root.name) / "Versions" / version_root.name


def python_archive_path(target: str, dependency_or_source: str, fallback_name: str) -> str:
    if is_darwin_target(target):
        return macos_python_archive_path(dependency_or_source, fallback_name)
    return f"lib/{fallback_name}"


def python_library_search_dirs() -> list[Path]:
    candidates: list[Path] = []

    def add(path: Path | None) -> None:
        if path and path.is_dir() and path not in candidates:
            candidates.append(path)

    for variable in ("LIBDIR", "LIBPL"):
        value = sysconfig.get_config_var(variable)
        if value:
            add(Path(value))

    base_exec_prefix = Path(sys.base_exec_prefix)
    add(base_exec_prefix / "lib")

    framework = sysconfig.get_config_var("PYTHONFRAMEWORK")
    framework_prefix = sysconfig.get_config_var("PYTHONFRAMEWORKPREFIX")
    framework_version = sysconfig.get_config_var("VERSION") or (
        f"{sys.version_info.major}.{sys.version_info.minor}"
    )
    if framework and framework_prefix:
        framework_root = Path(framework_prefix) / f"{framework}.framework" / "Versions" / framework_version
        add(framework_root)
        add(framework_root / "lib")

    return candidates


def python_shared_library_candidates(
    target: str, decode_runtime: Path
) -> list[tuple[Path, str]]:
    entries: dict[str, Path] = {}

    def add_candidate(path: Path, archive_path: str | None = None) -> None:
        if not path.is_file():
            return
        destination_path = archive_path or python_archive_path(target, str(path), path.name)
        if destination_path.endswith(".a"):
            return
        entries[destination_path] = path

    linked_dependencies = linked_python_dependencies(target, decode_runtime)
    for dependency, dependency_name in linked_dependencies:
        dependency_path = Path(dependency)
        if dependency_path.is_absolute():
            add_candidate(
                dependency_path,
                python_archive_path(target, dependency, dependency_name),
            )

    expected_names = {
        value
        for value in (
            sysconfig.get_config_var("INSTSONAME"),
            sysconfig.get_config_var("LDLIBRARY"),
        )
        if value
    }
    expected_names.update(name for _, name in linked_dependencies)

    for directory in python_library_search_dirs():
        for name in sorted(expected_names):
            add_candidate(
                directory / name,
                python_archive_path(target, str(directory / name), name),
            )

        for pattern in (
            f"libpython{sys.version_info.major}.{sys.version_info.minor}*.so*",
            f"libpython{sys.version_info.major}.{sys.version_info.minor}*.dylib*",
            "Python",
        ):
            for path in sorted(directory.glob(pattern)):
                if is_python_dependency(path.name):
                    add_candidate(path)

    return sorted((source, archive_path) for archive_path, source in entries.items())


def copy_file_to_staging(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source.resolve(), destination)


def macos_loader_path_reference(from_archive_path: PurePosixPath, to_archive_path: PurePosixPath) -> str:
    relative_path = posixpath.relpath(
        to_archive_path.as_posix(),
        from_archive_path.parent.as_posix(),
    )
    return f"@loader_path/{relative_path}"


def macos_framework_dependency_replacement(
    dependency: str,
    from_archive_path: PurePosixPath,
    framework_version_root: Path,
    framework_archive_version_root: PurePosixPath,
) -> str | None:
    dependency_path = Path(dependency)
    if not dependency_path.is_absolute():
        return None

    relative_dependency: PurePosixPath | None = None
    try:
        relative_dependency = PurePosixPath(
            dependency_path.relative_to(framework_version_root).as_posix()
        )
    except ValueError:
        framework_name = framework_version_root.parent.parent.name
        parts = dependency_path.parts
        for index, part in enumerate(parts):
            if (
                part == framework_name
                and index + 2 < len(parts)
                and parts[index + 1] == "Versions"
                and parts[index + 2] == framework_version_root.name
            ):
                relative_dependency = PurePosixPath(*parts[index + 3 :])
                break

    if relative_dependency is None:
        return None

    bundled_dependency = (
        PurePosixPath("python")
        / framework_archive_version_root
        / relative_dependency
    )
    return macos_loader_path_reference(from_archive_path, bundled_dependency)


def rewrite_macos_framework_references(
    file: Path,
    archive_path: PurePosixPath,
    framework_version_root: Path,
    framework_archive_version_root: PurePosixPath,
) -> None:
    install_name_tool = require_tool("install_name_tool")

    if file.name == "Python" or file.suffix == ".dylib":
        install_name = macos_loader_path_reference(archive_path, archive_path)
        subprocess.run([install_name_tool, "-id", install_name, str(file)], check=True)

    for dependency in macos_linked_libraries(file):
        replacement = macos_framework_dependency_replacement(
            dependency,
            archive_path,
            framework_version_root,
            framework_archive_version_root,
        )
        if not replacement or dependency == replacement:
            continue

        subprocess.run(
            [install_name_tool, "-change", dependency, replacement, str(file)],
            check=True,
        )


def macos_framework_support_libraries(version_root: Path) -> list[Path]:
    lib_dir = version_root / "lib"
    if not lib_dir.is_dir():
        return []
    return sorted(path for path in lib_dir.glob("*.dylib") if path.is_file() and not path.is_symlink())


def add_macos_framework_runtime_to_staging(
    staging_root: Path,
    version_root: Path,
) -> PurePosixPath:
    framework_archive_version_root = macos_framework_archive_version_root(version_root)
    framework_binary = version_root / "Python"
    ensure_file(framework_binary, "macOS Python framework binary")

    framework_binary_archive_path = (
        PurePosixPath("python") / framework_archive_version_root / "Python"
    )
    staged_framework_binary = staging_root / framework_binary_archive_path
    copy_file_to_staging(framework_binary, staged_framework_binary)
    rewrite_macos_framework_references(
        staged_framework_binary,
        framework_binary_archive_path,
        version_root,
        framework_archive_version_root,
    )

    for library in macos_framework_support_libraries(version_root):
        relative_library = library.relative_to(version_root)
        library_archive_path = (
            PurePosixPath("python")
            / framework_archive_version_root
            / PurePosixPath(relative_library.as_posix())
        )
        staged_library = staging_root / library_archive_path
        copy_file_to_staging(library, staged_library)
        rewrite_macos_framework_references(
            staged_library,
            library_archive_path,
            version_root,
            framework_archive_version_root,
        )

    return framework_archive_version_root


def rewrite_macos_stdlib_extension_references(
    staging_root: Path,
    stdlib_destination: Path,
    framework_version_root: Path,
    framework_archive_version_root: PurePosixPath,
) -> None:
    lib_dynload = stdlib_destination / "lib-dynload"
    if not lib_dynload.is_dir():
        return

    for extension in sorted(lib_dynload.glob("*.so")):
        extension_archive_path = PurePosixPath(
            (PurePosixPath("python") / extension.relative_to(staging_root / "python")).as_posix()
        )
        rewrite_macos_framework_references(
            extension,
            extension_archive_path,
            framework_version_root,
            framework_archive_version_root,
        )


def add_macos_python_runtime(
    archive: tarfile.TarFile,
    archive_root: str,
    shared_libraries: list[tuple[Path, str]],
) -> None:
    framework_versions: dict[Path, PurePosixPath] = {}

    with tempfile.TemporaryDirectory(prefix="dsview-macos-python-") as staging:
        staging_root = Path(staging)
        staged_python_root = staging_root / "python"

        for library, archive_path in shared_libraries:
            version_root = macos_framework_version_root(library)
            if version_root is not None and version_root.is_dir():
                if version_root not in framework_versions:
                    framework_versions[version_root] = add_macos_framework_runtime_to_staging(
                        staging_root,
                        version_root,
                    )
                continue

            copied_library = staged_python_root / PurePosixPath(archive_path)
            copy_file_to_staging(library, copied_library)

        stdlib_dir = Path(sysconfig.get_path("stdlib"))
        ensure_directory(stdlib_dir, "Python stdlib directory")
        stdlib_destination = staged_python_root / "lib" / python_version_dir()
        copy_directory_filtered(stdlib_dir, stdlib_destination, should_skip_python_path)

        platstdlib_value = sysconfig.get_path("platstdlib")
        if platstdlib_value:
            platstdlib_dir = Path(platstdlib_value)
            if (
                platstdlib_dir.is_dir()
                and platstdlib_dir.resolve() != stdlib_dir.resolve()
            ):
                copy_directory_filtered(
                    platstdlib_dir,
                    stdlib_destination,
                    should_skip_python_path,
                )

        for version_root, framework_archive_version_root in framework_versions.items():
            rewrite_macos_stdlib_extension_references(
                staging_root,
                stdlib_destination,
                version_root,
                framework_archive_version_root,
            )

        add_directory_filtered(
            archive,
            staged_python_root,
            f"{archive_root}/python",
            should_never_skip,
        )


def add_unix_python_runtime(
    archive: tarfile.TarFile,
    archive_root: str,
    target: str,
    decode_runtime: Path,
) -> None:
    shared_libraries = python_shared_library_candidates(target, decode_runtime)
    if not shared_libraries:
        raise FileNotFoundError(
            "No Unix Python shared runtime library was found to bundle"
        )

    required_library_paths = {
        python_archive_path(target, dependency, name)
        for dependency, name in linked_python_dependencies(target, decode_runtime)
    }
    bundled_library_paths = {archive_path for _, archive_path in shared_libraries}
    missing_library_paths = sorted(required_library_paths - bundled_library_paths)
    if missing_library_paths:
        raise FileNotFoundError(
            "Bundled Python runtime is missing linked libraries: "
            + ", ".join(missing_library_paths)
        )

    required_shared_libraries = [
        (library, archive_path)
        for library, archive_path in shared_libraries
        if archive_path in required_library_paths
    ]

    if is_darwin_target(target):
        add_macos_python_runtime(archive, archive_root, required_shared_libraries)
        return

    for library, archive_path in required_shared_libraries:
        add_regular_file(archive, library, f"{archive_root}/python/{archive_path}")

    stdlib_dir = Path(sysconfig.get_path("stdlib"))
    ensure_directory(stdlib_dir, "Python stdlib directory")
    stdlib_destination = f"{archive_root}/python/lib/{python_version_dir()}"
    add_directory_filtered(archive, stdlib_dir, stdlib_destination, should_skip_python_path)

    platstdlib_value = sysconfig.get_path("platstdlib")
    if platstdlib_value:
        platstdlib_dir = Path(platstdlib_value)
        if (
            platstdlib_dir.is_dir()
            and platstdlib_dir.resolve() != stdlib_dir.resolve()
        ):
            add_directory_filtered(
                archive,
                platstdlib_dir,
                stdlib_destination,
                should_skip_python_path,
            )


def prepare_macos_decode_runtime(source: Path, staging_dir: Path) -> Path:
    staged_runtime = staging_dir / source.name
    shutil.copy2(source, staged_runtime)

    python_dependencies = macos_linked_python_dependencies(source)
    if not python_dependencies:
        return staged_runtime

    install_name_tool = shutil.which("install_name_tool")
    if not install_name_tool:
        raise FileNotFoundError(
            "install_name_tool is required to make macOS Python runtime links bundle-relative"
        )

    for dependency, dependency_name in python_dependencies:
        archive_path = macos_python_archive_path(dependency, dependency_name)
        replacement = f"@loader_path/../python/{archive_path}"
        if dependency == replacement:
            continue
        subprocess.run(
            [install_name_tool, "-change", dependency, replacement, str(staged_runtime)],
            check=True,
        )

    return staged_runtime


def main() -> int:
    args = parse_args()

    ensure_file(args.exe, "Executable")
    ensure_file(args.runtime, "Runtime library")
    ensure_file(args.decode_runtime, "Decode runtime library")
    ensure_directory(args.resources, "Resources directory")
    ensure_directory(args.decoder_dir, "Decoder scripts directory")

    archive_root = f"dsview-cli-{args.version}-{args.target}"
    exe_name = "dsview-cli.exe" if is_windows_target(args.target) else "dsview-cli"

    required_resources = [
        "DSLogicPlus.fw",
        "DSLogic.fw",  # fallback resource kept for compatibility
        "DSLogicPlus.bin",
        "DSLogicPlus-pgl12.bin",
    ]

    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="dsview-package-") as staging:
        decode_runtime = args.decode_runtime
        if is_darwin_target(args.target):
            decode_runtime = prepare_macos_decode_runtime(
                args.decode_runtime,
                Path(staging),
            )

        with tarfile.open(args.output, "w:gz") as archive:
            add_file(archive, args.exe, f"{archive_root}/{exe_name}")
            add_file(archive, args.runtime, f"{archive_root}/runtime/{args.runtime.name}")
            add_file(
                archive,
                decode_runtime,
                f"{archive_root}/decode-runtime/{args.decode_runtime.name}",
            )
            add_directory(archive, args.decoder_dir, f"{archive_root}/decoders")
            if is_windows_target(args.target):
                for dependency in windows_dependency_dlls(args.target, args.runtime.name):
                    add_file(
                        archive,
                        dependency,
                        f"{archive_root}/{dependency.name}",
                    )
                add_windows_python_runtime(archive, archive_root)
            else:
                add_unix_python_runtime(
                    archive,
                    archive_root,
                    args.target,
                    args.decode_runtime,
                )

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
