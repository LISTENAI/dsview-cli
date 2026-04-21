use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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

    fn decode_runtime_library_name(&self) -> String {
        match self.os.as_str() {
            "windows" => "dsview_decode_runtime.dll".to_string(),
            "macos" => "libdsview_decode_runtime.dylib".to_string(),
            _ => "libdsview_decode_runtime.so".to_string(),
        }
    }

    fn needs_dl_link(&self) -> bool {
        matches!(self.os.as_str(), "linux" | "android")
    }

    fn is_windows_msvc(&self) -> bool {
        self.os == "windows" && self.env == "msvc"
    }

    fn windows_cmake_generator_platform(&self) -> Option<&'static str> {
        if !self.is_windows_msvc() {
            return None;
        }

        match self.arch.as_str() {
            "x86_64" => Some("x64"),
            "aarch64" => Some("ARM64"),
            _ => None,
        }
    }

    fn windows_vcpkg_triplet(&self) -> Option<&'static str> {
        if !self.is_windows_msvc() {
            return None;
        }

        match self.arch.as_str() {
            "x86_64" => Some("x64-windows"),
            "aarch64" => Some("arm64-windows"),
            _ => None,
        }
    }
}

struct SourceRuntimeArtifacts {
    capture_library_path: PathBuf,
    decode_library_path: PathBuf,
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
    let compat_root = manifest_dir.join("compat");

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
    println!(
        "cargo:rerun-if-changed={}",
        dsview_root.join("libsigrokdecode4DSL/libsigrokdecode.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        dsview_root.join("libsigrokdecode4DSL/srd.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        dsview_root.join("libsigrokdecode4DSL/decoder.c").display()
    );
    println!("cargo:rerun-if-changed={}", smoke_shim.display());
    println!("cargo:rerun-if-changed={}", runtime_bridge.display());
    println!(
        "cargo:rerun-if-changed={}",
        native_root.join("CMakeLists.txt").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        native_root.join("windows/input_minimal.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        native_root.join("windows/output_minimal.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        native_root.join("windows/session_stubs.c").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        native_root.join("windows/dsview_runtime.def").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        compat_root.join("msvc_preinclude.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        compat_root.join("pthread.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        compat_root.join("unistd.h").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        compat_root.join("sys/time.h").display()
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
    println!("cargo:rustc-check-cfg=cfg(dsview_source_decode_runtime_available)");
    println!("cargo:rustc-cfg=dsview_native_boundary");
    println!("cargo:rustc-cfg=dsview_runtime_bridge");
    println!("cargo:include={}", libsigrok_root.display());
    println!("cargo:include={}", common_root.display());
    println!(
        "cargo:rustc-env=DSVIEW_LIBSIGROK_HEADER={}",
        libsigrok_root.join("libsigrok.h").display()
    );

    let bridge_include_flags = bridge_dependency_include_flags(&target);
    build_static_object_archive(
        &runtime_bridge,
        "bridge_runtime",
        &[
            format!("-I{}", compat_root.display()),
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
                format!("-I{}", compat_root.display()),
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

    match build_source_runtimes(&dsview_repo_root, &native_root, &target) {
        Ok(artifacts) => {
            println!("cargo:rustc-cfg=dsview_source_runtime_available");
            println!(
                "cargo:rustc-env=DSVIEW_SOURCE_RUNTIME_LIBRARY={}",
                artifacts.capture_library_path.display()
            );
            println!("cargo:rustc-cfg=dsview_source_decode_runtime_available");
            println!(
                "cargo:rustc-env=DSVIEW_SOURCE_DECODE_RUNTIME_LIBRARY={}",
                artifacts.decode_library_path.display()
            );
            println!(
                "cargo:warning=Built source-backed DSView runtime at {}.",
                artifacts.capture_library_path.display()
            );
            println!(
                "cargo:warning=Built source-backed DSView decode runtime at {}.",
                artifacts.decode_library_path.display()
            );
        }
        Err(message) => {
            println!(
                "cargo:warning=Skipping source-backed DSView capture/decode runtime builds: {message}"
            );
        }
    }

    println!(
        "cargo:warning=dsview-sys is pinned to DSView/libsigrok4DSL headers and now exposes a narrow dynamic ds_* bring-up bridge."
    );
    println!(
        "cargo:warning=dsview-sys can use either a caller-supplied runtime library path or the locally built source runtime when native prerequisites are present."
    );
    println!(
        "cargo:warning=dsview-sys keeps decode discovery on a separate source-built runtime artifact so capture packaging does not inherit decoder prerequisites."
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

fn build_source_runtimes(
    repo_root: &Path,
    native_root: &Path,
    target: &TargetInfo,
) -> Result<SourceRuntimeArtifacts, String> {
    if !command_available("cmake") {
        return Err("cmake is not available".to_string());
    }
    if pkg_config_command().is_none() {
        return Err("pkg-config is not available".to_string());
    }

    let required_packages = ["glib-2.0", "libusb-1.0", "fftw3"];
    for package in required_packages {
        if !pkg_config_has(package) {
            return Err(format!("pkg-config could not resolve `{package}`"));
        }
    }

    if !command_available("python3") {
        return Err("python3 is not available".to_string());
    }

    let capture_library_path = build_source_runtime_variant(
        repo_root,
        native_root,
        target,
        "source-runtime-build",
        "capture",
    )?;
    let decode_library_path = build_source_runtime_variant(
        repo_root,
        native_root,
        target,
        "source-decode-runtime-build",
        "decode",
    )?;

    Ok(SourceRuntimeArtifacts {
        capture_library_path,
        decode_library_path,
    })
}

fn build_source_runtime_variant(
    repo_root: &Path,
    native_root: &Path,
    target: &TargetInfo,
    build_dir_name: &str,
    runtime_kind: &str,
) -> Result<PathBuf, String> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let build_dir = out_dir.join(build_dir_name);
    std::fs::create_dir_all(&build_dir).map_err(|error| {
        format!(
            "failed to create {runtime_kind} source runtime build directory: {error}"
        )
    })?;

    let mut configure = Command::new("cmake");
    configure
        .arg("-S")
        .arg(native_root)
        .arg("-B")
        .arg(&build_dir)
        .arg(format!("-DDSVIEW_REPO_ROOT={}", repo_root.display()));

    match runtime_kind {
        "capture" => {
            configure
                .arg("-DDSVIEW_BUILD_CAPTURE_RUNTIME=ON")
                .arg("-DDSVIEW_BUILD_DECODE_RUNTIME=OFF");
        }
        "decode" => {
            configure
                .arg("-DDSVIEW_BUILD_CAPTURE_RUNTIME=OFF")
                .arg("-DDSVIEW_BUILD_DECODE_RUNTIME=ON");
        }
        _ => return Err(format!("unknown source runtime kind `{runtime_kind}`")),
    }

    if let Some(platform) = target.windows_cmake_generator_platform() {
        configure.arg("-A").arg(platform);
    }
    if let Some(toolchain_file) = cmake_toolchain_file() {
        configure.arg(format!(
            "-DCMAKE_TOOLCHAIN_FILE={}",
            toolchain_file.display()
        ));
    }
    if let Some(triplet) = windows_vcpkg_triplet(target) {
        configure.arg(format!("-DVCPKG_TARGET_TRIPLET={triplet}"));
    }

    let configure_output = configure.output().map_err(|error| {
        format!(
            "failed to launch cmake configure for {runtime_kind} runtime: {error}"
        )
    })?;
    if !configure_output.status.success() {
        emit_command_failure_diagnostics("cmake configure", &configure, &configure_output);
        return Err(format!(
            "cmake configure failed for source-backed {runtime_kind} runtime"
        ));
    }
    }

    let mut build = Command::new("cmake");
    build.arg("--build").arg(&build_dir);
    if target.is_windows_msvc() {
        build.arg("--config").arg(cmake_build_config());
    }

    let build_output = build.output().map_err(|error| {
        format!(
            "failed to launch cmake build for {runtime_kind} runtime: {error}"
        )
    })?;
    if !build_output.status.success() {
        emit_command_failure_diagnostics("cmake build", &build, &build_output);
        return Err(format!(
            "cmake build failed for source-backed {runtime_kind} runtime"
        ));
    }
    }

    let library_name = match runtime_kind {
        "capture" => target.runtime_library_name(),
        "decode" => target.decode_runtime_library_name(),
        _ => unreachable!(),
    };
    let library_path = if target.is_windows_msvc() {
        build_dir.join(cmake_build_config()).join(&library_name)
    } else {
        build_dir.join(&library_name)
    };
    if !library_path.exists() {
        return Err(format!(
            "expected source-backed {runtime_kind} runtime artifact at {} (target: {}-{})",
            library_path.display(),
            target.os,
            target.arch
        ));
    }

    Ok(library_path)
}

fn pkg_config_output(package: &str, flag: &str) -> Vec<String> {
    let pkg_config = pkg_config_command().expect("pkg-config compatible command should exist");
    let output = Command::new(pkg_config)
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

fn bridge_dependency_include_flags(_target: &TargetInfo) -> Vec<String> {
    let mut flags = Vec::new();
    for package in ["glib-2.0", "libusb-1.0", "fftw3"] {
        for flag in pkg_config_output(package, "--cflags") {
            let parent_flag = if package == "libusb-1.0" {
                normalized_libusb_include_flag(&flag)
            } else {
                None
            };

            if !flags.contains(&flag) {
                flags.push(flag);
            }
            if let Some(parent_flag) = parent_flag {
                if !flags.contains(&parent_flag) {
                    flags.push(parent_flag);
                }
            }
        }
    }
    flags
}

fn emit_glib_link_flags(_target: &TargetInfo) {
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
    if target.is_windows_msvc() {
        let object_path = out_dir.join(format!("{archive_stem}.obj"));
        let archive_path = out_dir.join(format!("dsview_sys_{archive_stem}.lib"));
        let mut compile = Command::new("cl");
        compile
            .arg("/nologo")
            .arg("/c")
            .arg("/TC")
            .arg(source)
            .arg(format!("/Fo{}", object_path.display()));
        for include in include_flags {
            append_msvc_flag(&mut compile, include);
        }
        for flag in extra_flags {
            append_msvc_flag(&mut compile, flag);
        }

        let status = compile
            .status()
            .expect("failed to invoke MSVC compiler for dsview-sys shim");
        if !status.success() {
            panic!(
                "failed to compile dsview-sys shim source {} with cl.exe",
                source.display()
            );
        }

        let status = Command::new("lib")
            .arg("/nologo")
            .arg(format!("/OUT:{}", archive_path.display()))
            .arg(&object_path)
            .status()
            .expect("failed to invoke lib.exe for dsview-sys shim");
        if !status.success() {
            panic!(
                "failed to archive dsview-sys shim source {} with lib.exe",
                source.display()
            );
        }

        println!("cargo:rustc-link-search=native={}", out_dir.display());
        println!("cargo:rustc-link-lib=static=dsview_sys_{archive_stem}");
        println!(
            "cargo:warning=Built dsview-sys shim {}.",
            source.file_name().unwrap_or_default().to_string_lossy()
        );
        return;
    }

    let object_path = out_dir.join(format!("{archive_stem}.o"));
    let archive_path = out_dir.join(format!("libdsview_sys_{archive_stem}.a"));
    let mut compile = Command::new("cc");
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
    let Some(pkg_config) = pkg_config_command() else {
        return false;
    };

    Command::new(pkg_config)
        .arg("--exists")
        .arg(package)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn pkg_config_command() -> Option<String> {
    if let Ok(explicit) = env::var("PKG_CONFIG") {
        return Some(explicit);
    }

    for candidate in ["pkg-config", "pkgconf"] {
        if command_available(candidate) {
            return Some(candidate.to_string());
        }
    }

    None
}

fn cmake_toolchain_file() -> Option<PathBuf> {
    if let Ok(path) = env::var("CMAKE_TOOLCHAIN_FILE") {
        return Some(PathBuf::from(path));
    }

    vcpkg_root().map(|root| root.join("scripts/buildsystems/vcpkg.cmake"))
}

fn cmake_build_config() -> &'static str {
    match env::var("PROFILE").as_deref() {
        Ok("release") => "Release",
        _ => "Debug",
    }
}

fn windows_vcpkg_triplet(target: &TargetInfo) -> Option<String> {
    env::var("DSVIEW_VCPKG_TRIPLET")
        .ok()
        .or_else(|| env::var("VCPKG_TARGET_TRIPLET").ok())
        .or_else(|| target.windows_vcpkg_triplet().map(str::to_string))
}

fn vcpkg_root() -> Option<PathBuf> {
    env::var("DSVIEW_VCPKG_ROOT")
        .ok()
        .or_else(|| env::var("VCPKG_INSTALLATION_ROOT").ok())
        .or_else(|| env::var("VCPKG_ROOT").ok())
        .map(PathBuf::from)
}

fn append_msvc_flag(command: &mut Command, flag: &str) {
    if let Some(include) = flag.strip_prefix("-I") {
        command.arg(format!("/I{include}"));
    } else if let Some(define) = flag.strip_prefix("-D") {
        command.arg(format!("/D{define}"));
    } else {
        command.arg(flag);
    }
}

fn normalized_libusb_include_flag(flag: &str) -> Option<String> {
    let include_path = flag.strip_prefix("-I")?;
    let include_path = Path::new(include_path);
    if include_path.file_name()? != "libusb-1.0" {
        return None;
    }

    Some(format!("-I{}", include_path.parent()?.display()))
}

fn emit_command_failure_diagnostics(label: &str, command: &Command, output: &Output) {
    println!(
        "cargo:warning={} failed with status {} while building source-backed runtime.",
        label,
        output.status
    );
    println!("cargo:warning={} command: {}", label, command_debug_string(command));
    emit_command_stream(label, "stdout", &output.stdout);
    emit_command_stream(label, "stderr", &output.stderr);
}

fn emit_command_stream(label: &str, stream_name: &str, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    if text.trim().is_empty() {
        println!("cargo:warning={} {}: <empty>", label, stream_name);
        return;
    }

    for line in text.lines() {
        println!("cargo:warning={} {}: {}", label, stream_name, line);
    }
}

fn command_debug_string(command: &Command) -> String {
    let mut rendered = Vec::new();
    rendered.push(command.get_program().to_string_lossy().into_owned());
    rendered.extend(
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned()),
    );
    rendered.join(" ")
}
