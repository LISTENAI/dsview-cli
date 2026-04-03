//! Raw native integration boundary for DSView CLI.
//!
//! This crate is the only allowed home for unsafe FFI when Phase 1 adds
//! bindings to `DSView/libsigrok4DSL`.

use std::ffi::CStr;
#[cfg(dsview_smoke_runtime)]
use std::os::raw::c_char;

#[cfg(dsview_smoke_runtime)]
unsafe extern "C" {
    /// Public frontend symbol exported by `DSView/libsigrok4DSL`.
    pub fn sr_get_lib_version_string() -> *const c_char;
}

/// Reports whether the sys boundary is wired to the DSView public frontend API.
pub fn native_boundary_ready() -> bool {
    cfg!(dsview_native_boundary)
}

/// Reports whether the scoped runtime smoke shim is available on this machine.
pub fn runtime_smoke_ready() -> bool {
    cfg!(dsview_smoke_runtime)
}

/// Returns the public libsigrok4DSL version string when a native library is linked.
#[cfg(dsview_smoke_runtime)]
pub fn lib_version_string() -> Option<&'static CStr> {
    if !native_boundary_ready() {
        return None;
    }

    let raw = unsafe { sr_get_lib_version_string() };
    if raw.is_null() {
        None
    } else {
        Some(unsafe { CStr::from_ptr(raw) })
    }
}

/// Returns `None` when the scoped runtime smoke shim is unavailable.
#[cfg(not(dsview_smoke_runtime))]
pub fn lib_version_string() -> Option<&'static CStr> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_boundary_cfg_is_enabled() {
        assert!(native_boundary_ready(), "build script should enable the dsview_native_boundary cfg");
    }

    #[test]
    fn runtime_smoke_matches_environment() {
        let expected = std::path::Path::new("/usr/include/glib-2.0/glib.h").exists()
            && std::path::Path::new("/usr/lib/x86_64-linux-gnu/glib-2.0/include/glibconfig.h").exists();
        assert_eq!(runtime_smoke_ready(), expected, "runtime smoke availability should reflect whether glib development headers are present");
    }

    #[test]
    fn lib_version_smoke_returns_expected_version_when_runtime_is_available() {
        if runtime_smoke_ready() {
            let version = lib_version_string().expect("runtime smoke shim should return a version string");
            assert_eq!(version.to_str().unwrap(), "1.3.0");
        } else {
            assert!(lib_version_string().is_none(), "without the runtime smoke shim, version lookup should stay disabled");
        }
    }
}
