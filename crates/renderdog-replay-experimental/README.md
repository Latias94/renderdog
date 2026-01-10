# renderdog-replay-experimental

Experimental RenderDoc *replay* bindings via a small C++ shim and the `cxx` crate.

This crate is **not published** to crates.io (`publish = false`), and the API is expected to change.

See the [workspace README](../../README.md) for the stable crates and the MCP workflow.

## Status

- Goal: open an `.rdc` capture and expose a few replay operations (e.g. list textures, pick pixels, save textures).
- Approach: dynamically load the local RenderDoc library (`renderdoc.dll` / `librenderdoc.so`) and call replay APIs.

## Build

Enable the feature:

`cargo build -p renderdog-replay-experimental --features cxx-replay`

