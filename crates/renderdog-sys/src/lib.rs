// This crate provides low-level FFI bindings for RenderDoc's in-application API.
// Bindings are emitted to OUT_DIR/bindings.rs by build.rs.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
