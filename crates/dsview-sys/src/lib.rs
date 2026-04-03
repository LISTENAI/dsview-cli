//! Raw native integration boundary for DSView CLI.
//!
//! This crate is the only allowed home for unsafe FFI when Phase 1 adds
//! bindings to `DSView/libsigrok4DSL`.

/// Reports whether the sys boundary is linked to a native backend yet.
pub fn native_boundary_ready() -> bool {
    false
}
