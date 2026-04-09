use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Target descriptor for platform-aware build decisions.
struct TargetInfo {
    os: String,
    arch: String,
    env: String,
}

impl TargetInfo {
    fn from_cargo_env() -> Self {
        Self {
            os: env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS is set by Cargo"),
            arch: env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH is set by Cargo"),
            env: env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default(),
        }
    }

    fn runtime_library_name(&self) -> String {
        match self.os.as_str() {
            "windows" => "dsview_runtime.dll".to_string(),
            "macos" => "libdsview_runtime.dylib".to_string(),
            _ => "libdsview_runtime.so".to_string(),
        }
    }

    fn needs_dl_link(&self) -> bool {
        matches!(self.os.as_str(), "linux" | "android")
    }

    fn needs_m_link(&self) -> bool {
        !matches!(self.os.as_str(), "windows")
    }

    fn is_windows_msvc(&self) -> bool {
        self.os == "windows" && self.env == "msvc"
    }
}

fn main() {
    let target = TargetInfo::from_cargo_env();
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by Cargo"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("dsview-sys should live under <repo>/crates/dsview-sys")
        .to_path_buf();
    let dsview_repo_root =
        resolve_dsview_repo_root(&repo_root).unwrap_or_else(|| panic!("{}", missing_dsview_message(&repo_root)));
    let dsview_root = dsview_repo_root.join("DSView");
    let libsigrok_root = dsview_root.join("libsigrok4DSL");
    let common_root = dsview_root.join("common");
    let smoke_shim = manifest_dir.join("smoke_version_shim.c");
    let runtime_bridge = manifest_dir.join("bridge_runtime.c");
    let wrapper_header = manifest_dir.join("wrapper.h");
    let native_root = manifest_dir.join("native");

    println!("cargo:rerun-if-changed={}", wrapper_header.display());
    println!(
        "cargo:rerun-if-changed={}",
        dsview_root.join("CMakeLists.txt").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libsigrok_root.join("libsigrok.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libsigrok_root.join("output/output.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libsigrok_root.join("output/vcd.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libsigrok_root.join("dsdevice.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libsigrok_root.join("libsigrok-internal.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        libsigrok_root.join("config.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        dsview_root.join("common/ds_types.h").display()
    );
    println!("cargo:rerun-if-changed={}", smoke_shim.display());
    println!("cargo:rerun-if-changed={}", runtime_bridge.display());
    println!(
        "cargo:rerun-if-changed={}",
        native_root.join("CMakeLists.txt").display()
    );

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
    println!(
        "cargo:rustc-env=DSVIEW_LIBSIGROK_HEADER={}",
        libsigrok_root.join("libsigrok.h").display()
    );

    let bridge_include_flags = glib_include_flags(&target);
    build_static_object_archive(
        &runtime_bridge,
        "bridge_runtime",
        &[
            format!("-I{}", dsview_root.display()),
            format!("-I{}", libsigrok_root.display()),
            format!("-I{}", common_root.display()),
            format!("-I{}", manifest_dir.display()),
            format!("-I{}", native_root.display()),
        ],
        &bridge_include_flags,
        &target,
    );
    emit_glib_link_flags(&target);
    if target.needs_dl_link() {
        println!("cargo:rustc-link-lib=dl");
    }
    println!("cargo:warning=Built dsview-sys runtime bridge shim for dynamic ds_* loading.");

    if should_build_smoke_runtime(&target) {
        build_static_object_archive(
            &smoke_shim,
            "smoke_version",
            &[
                format!("-I{}", dsview_root.display()),
                format!("-I{}", libsigrok_root.display()),
                format!("-I{}", common_root.display()),
                format!("-I{}", manifest_dir.display()),
            ],
            &[],
            &target,
        );
        println!("cargo:rustc-cfg=dsview_runtime_smoke_available");
        println!(
            "cargo:warning=Built dsview-sys runtime smoke shim for sr_get_lib_version_string() using DSView/libsigrok4DSL/version.h."
        );
    } else {
        println!(
            "cargo:warning=Skipping dsview-sys runtime smoke shim because the environment is missing glib development headers."
        );
    }

    match build_source_runtime(&dsview_repo_root, &native_root, &target) {
        Ok(library_path) => {
            println!("cargo:rustc-cfg=dsview_source_runtime_available");
            println!(
                "cargo:rustc-env=DSVIEW_SOURCE_RUNTIME_LIBRARY={}",
                library_path.display()
            );
            println!(
                "cargo:warning=Built source-backed DSView runtime at {}.",
                library_path.display()
            );
        }
        Err(message) => {
            println!("cargo:warning=Skipping source-backed DSView runtime build: {message}");
        }
    }

    println!(
        "cargo:warning=dsview-sys is pinned to DSView/libsigrok4DSL headers and now exposes a narrow dynamic ds_* bring-up bridge."
    );
    println!(
        "cargo:warning=dsview-sys can use either a caller-supplied runtime library path or the locally built source runtime when native prerequisites are present."
    );
}

fn resolve_dsview_repo_root(repo_root: &Path) -> Option<PathBuf> {
    repo_root
        .ancestors()
        .find(|candidate| candidate.join("DSView/libsigrok4DSL/libsigrok.h").exists())
        .map(Path::to_path_buf)
}

fn missing_dsview_message(repo_root: &Path) -> String {
    format!(
        "DSView submodule not found from {}. Run `git submodule update --init --recursive` before building dsview-sys.",
        repo_root.display()
    )
}

fn should_build_smoke_runtime(target: &TargetInfo) -> bool {
    if target.is_windows_msvc() {
        return false;
    }
    header_exists("/usr/include/glib-2.0/glib.h")
        && header_exists("/usr/lib/x86_64-linux-gnu/glib-2.0/include/glibconfig.h")
        && command_available("cc")
        && command_available("ar")
}

fn build_source_runtime(repo_root: &Path, native_root: &Path, target: &TargetInfo) -> Result<PathBuf, String> {
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

    let library_path = build_dir.join(target.runtime_library_name());
    if !library_path.exists() {
        return Err(format!(
            "expected source runtime artifact at {} (target: {}-{})",
            library_path.display(),
            target.os,
            target.arch
        ));
    }

    Ok(library_path)
}

fn pkg_config_output(package: &str, flag: &str) -> Vec<String> {
    let output = Command::new("pkg-config")
        .arg(flag)
        .arg(package)
        .output()
        .unwrap_or_else(|error| panic!("failed to run pkg-config {flag} {package}: {error}"));
    if !output.status.success() {
        panic!("pkg-config {flag} {package} failed");
    }

    String::from_utf8(output.stdout)
        .expect("pkg-config output should be utf-8")
        .split_whitespace()
        .map(|value| value.to_string())
        .collect()
}

fn glib_include_flags(target: &TargetInfo) -> Vec<String> {
    if target.is_windows_msvc() {
        return vec![];
    }
    pkg_config_output("glib-2.0", "--cflags")
}

fn emit_glib_link_flags(target: &TargetInfo) {
    if target.is_windows_msvc() {
        return;
    }
    for flag in pkg_config_output("glib-2.0", "--libs") {
        if let Some(path) = flag.strip_prefix("-L") {
            println!("cargo:rustc-link-search=native={path}");
        } else if let Some(lib) = flag.strip_prefix("-l") {
            println!("cargo:rustc-link-lib={lib}");
        } else {
            println!(
                "cargo:warning=Ignoring unsupported glib link flag `{flag}` for dsview-sys bridge build."
            );
        }
    }
}

fn build_static_object_archive(
    source: &Path,
    archive_stem: &str,
    include_flags: &[String],
    extra_flags: &[String],
    target: &TargetInfo,
) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let object_path = out_dir.join(format!("{archive_stem}.o"));
    let archive_path = out_dir.join(format!("libdsview_sys_{archive_stem}.a"));

    let compiler = if target.is_windows_msvc() {
        "cl"
    } else {
        "cc"
    };

    if target.is_windows_msvc() {
        panic!(
            "MSVC compilation for dsview-sys shims is not yet implemented. \
             Target: {}-{}-{}",
            target.os, target.arch, target.env
        );
    }

    let mut compile = Command::new(compiler);
    compile.arg("-c").arg(source).arg("-o").arg(&object_path);
    for include in include_flags {
        compile.arg(include);
    }
    for flag in extra_flags {
        compile.arg(flag);
    }

    let status = compile
        .status()
        .expect("failed to invoke compiler for dsview-sys shim");
    if !status.success() {
        panic!(
            "failed to compile dsview-sys shim source {}",
            source.display()
        );
    }

    let status = Command::new("ar")
        .arg("crus")
        .arg(&archive_path)
        .arg(&object_path)
        .status()
        .expect("failed to invoke ar for dsview-sys shim");
    if !status.success() {
        panic!(
            "failed to archive dsview-sys shim source {}",
            source.display()
        );
    }

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dsview_sys_{archive_stem}");
    println!(
        "cargo:warning=Built dsview-sys shim {}.",
        source.file_name().unwrap_or_default().to_string_lossy()
    );
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
