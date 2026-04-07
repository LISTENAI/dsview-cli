use std::path::Path;

use dsview_sys::{source_runtime_library_path, upstream_header_path};

#[test]
fn upstream_header_exists_for_boundary_tests() {
    let header = upstream_header_path();
    assert!(header.ends_with("DSView/libsigrok4DSL/libsigrok.h"));
    assert!(header.exists(), "expected upstream header at {}", header.display());
}

#[test]
fn source_runtime_path_shape_matches_cfg_state() {
    if let Some(path) = source_runtime_library_path() {
        assert!(Path::new(path).is_absolute());
        assert!(path.to_string_lossy().ends_with("libdsview_runtime.so"));
    }
}
