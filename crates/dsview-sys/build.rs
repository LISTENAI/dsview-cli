use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("dsview-sys should live under <repo>/crates/dsview-sys")
        .to_path_buf();
    let dsview_root = repo_root.join("DSView");
    let libsigrok_root = dsview_root.join("libsigrok4DSL");
    let common_root = dsview_root.join("common");
    let smoke_shim = manifest_dir.join("smoke_version_shim.c");
    let runtime_bridge = manifest_dir.join("bridge_runtime.c");
    let wrapper_header = manifest_dir.join("wrapper.h");
    let native_root = manifest_dir.join("native");

    println!("cargo:rerun-if-changed={}", wrapper_header.display());
    println!("cargo:rerun-if-changed={}", dsview_root.join("CMakeLists.txt").display());
    println!("cargo:rerun-if-changed={}", libsigrok_root.join("libsigrok.h").display());
    println!("cargo:rerun-if-changed={}", libsigrok_root.join("version.c").display());
    println!("cargo:rerun-if-changed={}", libsigrok_root.join("version.h").display());
    println!("cargo:rerun-if-changed={}", smoke_shim.display());
    println!("cargo:rerun-if-changed={}", runtime_bridge.display());
    println!("cargo:rerun-if-changed={}", native_root.join("CMakeLists.txt").display());

    if !dsview_root.exists() {
        panic!(
            "DSView submodule not found at {}. Run `git submodule update --init --recursive` before building dsview-sys.",
            dsview_root.display()
        );
    }

    if !libsigrok_root.join("libsigrok.h").exists() {
        panic!(
            "Missing libsigrok4DSL public header at {}. Re-sync the DSView submodule before building dsview-sys.",
            libsigrok_root.join("libsigrok.h").display()
        );
    }

    println!("cargo:rustc-check-cfg=cfg(dsview_native_boundary)");
    println!("cargo:rustc-check-cfg=cfg(dsview_runtime_bridge)");
    println!("cargo:rustc-check-cfg=cfg(dsview_runtime_smoke_available)");
    println!("cargo:rustc-check-cfg=cfg(dsview_source_runtime_available)");
    println!("cargo:rustc-cfg=dsview_native_boundary");
    println!("cargo:rustc-cfg=dsview_runtime_bridge");
    println!("cargo:include={}", libsigrok_root.display());
    println!("cargo:include={}", common_root.display());
    println!("cargo:rustc-env=DSVIEW_LIBSIGROK_HEADER={}", libsigrok_root.join("libsigrok.h").display());

    build_static_object_archive(&runtime_bridge, "bridge_runtime", &[format!("-I{}", manifest_dir.display())]);
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:warning=Built dsview-sys runtime bridge shim for dynamic ds_* loading.");

    if should_build_smoke_runtime() {
        build_static_object_archive(
            &smoke_shim,
            "smoke_version",
            &[
                format!("-I{}", libsigrok_root.display()),
                format!("-I{}", manifest_dir.display()),
            ],
        );
        println!("cargo:rustc-cfg=dsview_runtime_smoke_available");
        println!("cargo:warning=Built dsview-sys runtime smoke shim for sr_get_lib_version_string() using DSView/libsigrok4DSL/version.h.");
    } else {
        println!("cargo:warning=Skipping dsview-sys runtime smoke shim because the environment is missing glib development headers.");
    }

    match build_source_runtime(&repo_root, &native_root) {
        Ok(library_path) => {
            println!("cargo:rustc-cfg=dsview_source_runtime_available");
            println!("cargo:rustc-env=DSVIEW_SOURCE_RUNTIME_LIBRARY={}", library_path.display());
            println!("cargo:warning=Built source-backed DSView runtime at {}.", library_path.display());
        }
        Err(message) => {
            println!("cargo:warning=Skipping source-backed DSView runtime build: {message}");
        }
    }

    println!("cargo:warning=dsview-sys is pinned to DSView/libsigrok4DSL headers and now exposes a narrow dynamic ds_* bring-up bridge.");
    println!("cargo:warning=dsview-sys can use either a caller-supplied runtime library path or the locally built source runtime when native prerequisites are present.");
}

fn should_build_smoke_runtime() -> bool {
    header_exists("/usr/include/glib-2.0/glib.h")
        && header_exists("/usr/lib/x86_64-linux-gnu/glib-2.0/include/glibconfig.h")
        && command_available("cc")
        && command_available("ar")
}

fn build_source_runtime(repo_root: &Path, native_root: &Path) -> Result<PathBuf, String> {
    if !command_available("cmake") {
        return Err("cmake is not available".to_string());
    }
    if !command_available("pkg-config") {
        return Err("pkg-config is not available".to_string());
    }

    let required_packages = ["glib-2.0", "libusb-1.0", "fftw3", "zlib"];
    for package in required_packages {
        if !pkg_config_has(package) {
            return Err(format!("pkg-config could not resolve `{package}`"));
        }
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let build_dir = out_dir.join("source-runtime-build");
    std::fs::create_dir_all(&build_dir)
        .map_err(|error| format!("failed to create source runtime build directory: {error}"))?;

    let configure_status = Command::new("cmake")
        .arg("-S")
        .arg(native_root)
        .arg("-B")
        .arg(&build_dir)
        .arg(format!("-DDSVIEW_REPO_ROOT={}", repo_root.display()))
        .status()
        .map_err(|error| format!("failed to launch cmake configure: {error}"))?;
    if !configure_status.success() {
        return Err("cmake configure failed for source-backed runtime".to_string());
    }

    let build_status = Command::new("cmake")
        .arg("--build")
        .arg(&build_dir)
        .status()
        .map_err(|error| format!("failed to launch cmake build: {error}"))?;
    if !build_status.success() {
        return Err("cmake build failed for source-backed runtime".to_string());
    }

    let library_path = build_dir.join("libdsview_runtime.so");
    if !library_path.exists() {
        return Err(format!("expected source runtime artifact at {}", library_path.display()));
    }

    Ok(library_path)
}

fn build_static_object_archive(source: &Path, archive_stem: &str, include_flags: &[String]) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let object_path = out_dir.join(format!("{archive_stem}.o"));
    let archive_path = out_dir.join(format!("libdsview_sys_{archive_stem}.a"));

    let mut compile = Command::new("cc");
    compile.arg("-c").arg(source).arg("-o").arg(&object_path);
    for include in include_flags {
        compile.arg(include);
    }

    let status = compile.status().expect("failed to invoke cc for dsview-sys shim");
    if !status.success() {
        panic!("failed to compile dsview-sys shim source {}", source.display());
    }

    let status = Command::new("ar")
        .arg("crus")
        .arg(&archive_path)
        .arg(&object_path)
        .status()
        .expect("failed to invoke ar for dsview-sys shim");
    if !status.success() {
        panic!("failed to archive dsview-sys shim source {}", source.display());
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dsview_sys_{archive_stem}");
    println!("cargo:warning=Built dsview-sys shim {}.", source.file_name().unwrap_or_default().to_string_lossy());
}

fn header_exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

fn command_available(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn pkg_config_has(package: &str) -> bool {
    Command::new("pkg-config")
        .arg("--exists")
        .arg(package)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
