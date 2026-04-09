use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use dsview_core::RuntimeDiscoveryPaths;
use dsview_sys::source_runtime_library_path;

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

#[test]
fn bundle_defaults_resolve_from_executable_layout() {
    let exe_dir = temp_dir("bundle-defaults");
    let runtime_dir = exe_dir.join("runtime");
    let resource_dir = exe_dir.join("resources");
    fs::create_dir_all(&runtime_dir).unwrap();
    fs::write(
        runtime_dir.join(platform_runtime_library_name()),
        b"runtime",
    )
    .unwrap();
    write_valid_resources(&resource_dir);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>)
        .expect("bundle-relative discovery should succeed");

    assert_eq!(paths.runtime_library, runtime_dir.join(platform_runtime_library_name()));
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
        runtime_dir.join(platform_runtime_library_name()),
        b"runtime",
    )
    .unwrap();
    write_valid_resources(&bundled_resources);
    write_valid_resources(&override_resources);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, Some(&override_resources))
        .expect("explicit resource override should succeed");

    assert_eq!(paths.runtime_library, runtime_dir.join(platform_runtime_library_name()));
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
        runtime_dir.join(platform_runtime_library_name()),
        b"runtime",
    )
    .unwrap();
    write_valid_resources(&resource_dir);

    let paths = RuntimeDiscoveryPaths::from_executable_dir(&exe_dir, None::<&std::path::Path>)
        .expect("bundle-relative defaults should resolve");

    assert_eq!(paths.resource_dir, resource_dir);
    assert_eq!(paths.runtime_library, runtime_dir.join(platform_runtime_library_name()));
}

fn platform_runtime_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "dsview_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libdsview_runtime.dylib"
    } else {
        "libdsview_runtime.so"
    }
}
