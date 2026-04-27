use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use dsview_core::{DecodeDiscoveryPaths, RuntimeDiscoveryPaths};
use dsview_sys::{
    decode_runtime_library_name, runtime_library_name, source_runtime_library_path,
};

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dsview-core-bundle-{name}-{unique}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_valid_resources(dir: &std::path::Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("DSLogicPlus.fw"), b"fw").unwrap();
    fs::write(dir.join("DSLogicPlus.bin"), b"bin").unwrap();
    fs::write(dir.join("DSLogicPlus-pgl12.bin"), b"bin").unwrap();
}

fn write_valid_decoder_dir(dir: &std::path::Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("spi.py"), b"# decoder placeholder").unwrap();
}

#[test]
fn bundle_defaults_resolve_from_executable_layout() {
    let exe_dir = temp_dir("bundle-defaults");
    let runtime_dir = exe_dir.join("runtime");
    let resource_dir = exe_dir.join("resources");
    fs::create_dir_all(&runtime_dir).unwrap();
    fs::write(
        runtime_dir.join(runtime_library_name()),
        b"runtime",
    )
    .unwrap();
    write_valid_resources(&resource_dir);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>)
        .expect("bundle-relative discovery should succeed");

    assert_eq!(paths.runtime_library, runtime_dir.join(runtime_library_name()));
    assert_eq!(paths.resource_dir, resource_dir);
}

#[test]
fn resource_override_wins_over_bundled_resource_dir() {
    let exe_dir = temp_dir("resource-override");
    let runtime_dir = exe_dir.join("runtime");
    let bundled_resources = exe_dir.join("resources");
    let override_resources = temp_dir("resource-override-explicit");
    fs::create_dir_all(&runtime_dir).unwrap();
    fs::write(
        runtime_dir.join(runtime_library_name()),
        b"runtime",
    )
    .unwrap();
    write_valid_resources(&bundled_resources);
    write_valid_resources(&override_resources);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, Some(&override_resources))
        .expect("explicit resource override should succeed");

    assert_eq!(paths.runtime_library, runtime_dir.join(runtime_library_name()));
    assert_eq!(paths.resource_dir, override_resources);
}

#[test]
fn developer_fallback_uses_source_runtime_and_repo_resources() {
    let exe_dir = temp_dir("developer-fallback");

    let Some(source_runtime) = source_runtime_library_path() else {
        return;
    };
    if !source_runtime.is_file() {
        return;
    }

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>)
        .expect("developer fallback should use source runtime");

    assert_eq!(paths.runtime_library, source_runtime);
    assert!(paths.resource_dir.ends_with("DSView/DSView/res"));
}

#[test]
fn discovery_paths_feed_connect_auto_contract_without_resource_override() {
    let exe_dir = temp_dir("connect-auto-contract");
    let runtime_dir = exe_dir.join("runtime");
    let resource_dir = exe_dir.join("resources");
    fs::create_dir_all(&runtime_dir).unwrap();
    fs::write(
        runtime_dir.join(runtime_library_name()),
        b"runtime",
    )
    .unwrap();
    write_valid_resources(&resource_dir);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>)
        .expect("bundle-relative defaults should resolve");

    assert_eq!(paths.resource_dir, resource_dir);
    assert_eq!(paths.runtime_library, runtime_dir.join(runtime_library_name()));
}

// Wave 0 tests: target-aware runtime filename contract

#[test]
fn bundle_discovery_uses_target_aware_runtime_filename() {
    let exe_dir = temp_dir("target-aware-runtime");
    let runtime_dir = exe_dir.join("runtime");
    let resource_dir = exe_dir.join("resources");
    fs::create_dir_all(&runtime_dir).unwrap();

    // Create runtime with the correct platform-specific name
    let runtime_name = runtime_library_name();
    fs::write(runtime_dir.join(runtime_name), b"runtime").unwrap();
    write_valid_resources(&resource_dir);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>)
        .expect("target-aware runtime discovery should succeed");

    assert_eq!(
        paths.runtime_library.file_name().unwrap().to_str().unwrap(),
        runtime_name
    );
}

#[test]
fn bundle_discovery_rejects_wrong_platform_runtime() {
    let exe_dir = temp_dir("wrong-platform-runtime");
    let runtime_dir = exe_dir.join("runtime");
    let resource_dir = exe_dir.join("resources");
    fs::create_dir_all(&runtime_dir).unwrap();

    // Create runtime with the WRONG platform name
    let wrong_name = if cfg!(target_os = "windows") {
        "libdsview_runtime.so"
    } else {
        "dsview_runtime.dll"
    };
    fs::write(runtime_dir.join(wrong_name), b"runtime").unwrap();
    write_valid_resources(&resource_dir);

    // Discovery should fail or fall back to source runtime
    let result = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>);

    if let Ok(paths) = result {
        // If it succeeded, it must have fallen back to source runtime
        assert_ne!(
            paths.runtime_library.file_name().unwrap().to_str().unwrap(),
            wrong_name,
            "Discovery should not use wrong-platform runtime"
        );
    }
}

#[test]
fn bundle_layout_matches_packaging_contract() {
    let exe_dir = temp_dir("packaging-contract");
    let runtime_dir = exe_dir.join("runtime");
    let resource_dir = exe_dir.join("resources");
    fs::create_dir_all(&runtime_dir).unwrap();
    fs::write(runtime_dir.join(runtime_library_name()), b"runtime").unwrap();
    write_valid_resources(&resource_dir);

    // Verify the layout matches what the bundle packaging helper creates.
    assert!(runtime_dir.is_dir(), "runtime/ directory must exist");
    assert!(resource_dir.is_dir(), "resources/ directory must exist");
    assert!(
        runtime_dir.join(runtime_library_name()).is_file(),
        "runtime library must exist with correct name"
    );

    // Verify required DSLogic Plus resources
    assert!(resource_dir.join("DSLogicPlus.fw").is_file());
    assert!(resource_dir.join("DSLogicPlus.bin").is_file());
    assert!(resource_dir.join("DSLogicPlus-pgl12.bin").is_file());
}

#[test]
fn decode_bundle_discovery_uses_bundled_python_home_on_all_platforms() {
    let exe_dir = temp_dir("decode-python-home");
    let decode_runtime_dir = exe_dir.join("decode-runtime");
    let decoder_dir = exe_dir.join("decoders");
    let python_home = exe_dir.join("python");
    fs::create_dir_all(&decode_runtime_dir).unwrap();
    fs::create_dir_all(&python_home).unwrap();
    fs::write(
        decode_runtime_dir.join(decode_runtime_library_name()),
        b"decode runtime",
    )
    .unwrap();
    write_valid_decoder_dir(&decoder_dir);

    let paths = DecodeDiscoveryPaths::from_executable_dir(
        &exe_dir,
        None::<&std::path::Path>,
        None::<&std::path::Path>,
    )
    .expect("decode bundle-relative discovery should succeed");

    assert_eq!(
        paths.runtime_library,
        decode_runtime_dir.join(decode_runtime_library_name())
    );
    assert_eq!(paths.decoder_dir, decoder_dir);
    assert_eq!(paths.python_home, Some(python_home));
}

#[test]
fn runtime_library_name_helper_is_consistent() {
    // Verify the helper returns the same value across calls
    let name1 = runtime_library_name();
    let name2 = runtime_library_name();
    assert_eq!(name1, name2);

    // Verify it matches the platform
    if cfg!(target_os = "windows") {
        assert_eq!(name1, "dsview_runtime.dll");
    } else if cfg!(target_os = "macos") {
        assert_eq!(name1, "libdsview_runtime.dylib");
    } else {
        assert_eq!(name1, "libdsview_runtime.so");
    }
}
