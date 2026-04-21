//! Low-level FFI bindings for RenderDoc's in-application API (`renderdoc_app.h`).
//!
//! This crate ships with pregenerated bindings for docs.rs and for environments where `bindgen`
//! (libclang) is not available. At build time, `build.rs` writes `OUT_DIR/bindings.rs` which is
//! then included by this crate.
//!
//! Maintainers can regenerate bindings with:
//!
//! - `RENDERDOG_SYS_REGEN_BINDINGS=1 cargo build -p renderdog-sys --features bindgen`
//! - or `python scripts/regen_bindings.py` from the workspace root
//!
//! Regeneration requires either the workspace `third-party/renderdoc` submodule or
//! `RENDERDOG_SYS_HEADER=/path/to/renderdoc_app.h`.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::all)]

mod version;

pub use version::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
