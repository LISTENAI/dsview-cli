use std::env;
use std::path::PathBuf;
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

    println!("cargo:rerun-if-changed={}", manifest_dir.join("wrapper.h").display());
    println!("cargo:rerun-if-changed={}", dsview_root.join("CMakeLists.txt").display());
    println!("cargo:rerun-if-changed={}", libsigrok_root.join("libsigrok.h").display());
    println!("cargo:rerun-if-changed={}", libsigrok_root.join("version.c").display());
    println!("cargo:rerun-if-changed={}", libsigrok_root.join("version.h").display());
    println!("cargo:rerun-if-changed={}", smoke_shim.display());

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
    println!("cargo:rustc-check-cfg=cfg(dsview_smoke_runtime)");
    println!("cargo:rustc-cfg=dsview_native_boundary");
    println!("cargo:include={}", libsigrok_root.display());
    println!("cargo:include={}", common_root.display());

    if should_build_smoke_runtime() {
        build_smoke_runtime(&manifest_dir, &libsigrok_root, &smoke_shim);
    } else {
        println!("cargo:warning=Skipping dsview-sys runtime smoke shim because the environment is missing glib development headers.");
    }

    println!("cargo:warning=dsview-sys is pinned to DSView/libsigrok4DSL headers and currently validates the public frontend boundary via sr_get_lib_version_string().");
    println!("cargo:warning=No standalone libsigrok4DSL artifact was found in-tree; later phases must either provide a buildable native library path or introduce a tiny shim without coupling to the DSView GUI target.");
}

fn should_build_smoke_runtime() -> bool {
    header_exists("/usr/include/glib-2.0/glib.h")
        && header_exists("/usr/lib/x86_64-linux-gnu/glib-2.0/include/glibconfig.h")
        && command_available("cc")
}

fn build_smoke_runtime(manifest_dir: &PathBuf, libsigrok_root: &PathBuf, smoke_shim: &PathBuf) {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let object_path = out_dir.join("smoke_version_shim.o");
    let archive_path = out_dir.join("libdsview_sys_smoke_version.a");

    let status = Command::new("cc")
        .arg("-c")
        .arg(smoke_shim)
        .arg("-o")
        .arg(&object_path)
        .arg(format!("-I{}", libsigrok_root.display()))
        .arg(format!("-I{}", manifest_dir.display()))
        .status()
        .expect("failed to invoke cc for dsview-sys smoke shim");

    if !status.success() {
        panic!("failed to compile dsview-sys smoke shim");
    }

    let status = Command::new("ar")
        .arg("crus")
        .arg(&archive_path)
        .arg(&object_path)
        .status()
        .expect("failed to invoke ar for dsview-sys smoke shim");

    if !status.success() {
        panic!("failed to archive dsview-sys smoke shim");
    }

    println!("cargo:rustc-cfg=dsview_smoke_runtime");
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dsview_sys_smoke_version");
    println!("cargo:warning=Built dsview-sys runtime smoke shim for sr_get_lib_version_string() using DSView/libsigrok4DSL/version.h.");
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
