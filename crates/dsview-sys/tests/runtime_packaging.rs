//! Wave 0 tests for target-aware runtime naming and packaging contract.
//!
//! These tests verify that the runtime library naming helper correctly maps
//! target platforms to their expected shared library filenames, ensuring
//! consistency between build.rs, packaging helpers, and runtime discovery.

use dsview_sys::runtime_library_name;

#[test]
fn runtime_library_name_matches_current_platform() {
    let name = runtime_library_name();

    if cfg!(target_os = "windows") {
        assert_eq!(name, "dsview_runtime.dll");
    } else if cfg!(target_os = "macos") {
        assert_eq!(name, "libdsview_runtime.dylib");
    } else {
        assert_eq!(name, "libdsview_runtime.so");
    }
}

#[test]
fn runtime_library_name_is_stable() {
    // The naming contract must remain stable for packaging and discovery
    let name1 = runtime_library_name();
    let name2 = runtime_library_name();
    assert_eq!(name1, name2);
}

#[test]
fn runtime_library_name_has_no_path_separators() {
    let name = runtime_library_name();
    assert!(!name.contains('/'));
    assert!(!name.contains('\\'));
}

#[test]
fn linux_runtime_naming() {
    #[cfg(target_os = "linux")]
    {
        assert_eq!(runtime_library_name(), "libdsview_runtime.so");
    }
}

#[test]
fn macos_runtime_naming() {
    #[cfg(target_os = "macos")]
    {
        assert_eq!(runtime_library_name(), "libdsview_runtime.dylib");
    }
}

#[test]
fn windows_runtime_naming() {
    #[cfg(target_os = "windows")]
    {
        assert_eq!(runtime_library_name(), "dsview_runtime.dll");
    }
}

#[test]
fn runtime_library_name_is_valid_filename() {
    let name = runtime_library_name();

    // Must not be empty
    assert!(!name.is_empty());

    // Must have an extension
    assert!(name.contains('.'));

    // Must not contain invalid characters
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
    for ch in invalid_chars {
        assert!(!name.contains(ch), "Runtime library name contains invalid character: {}", ch);
    }
}
