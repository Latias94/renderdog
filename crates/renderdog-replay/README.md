# renderdog-replay

Experimental RenderDoc *replay* bindings via a small C++ shim and the `cxx` crate.

This crate is **not published** to crates.io (`publish = false`), and the API is expected to change.

See the [workspace README](../../README.md) for the stable crates and the MCP workflow.

## Status

- Goal: open an `.rdc` capture and expose a few replay operations (e.g. list textures, pick pixels, save textures).
- Approach: dynamically load the local RenderDoc library (`renderdoc.dll` / `librenderdoc.so`) and call replay APIs.

## Build

Enable the feature:

`cargo build -p renderdog-replay --features cxx-replay`

## Runtime prerequisites

This crate dynamically loads the local RenderDoc library for replay:

- Windows: `renderdoc.dll`
- Linux: `librenderdoc.so` / `librenderdoc.so.1`

If it cannot be found via the OS loader, set one of:

- `RENDERDOG_REPLAY_RENDERDOC_DLL` (Windows, full path to `renderdoc.dll`)
- `RENDERDOG_REPLAY_RENDERDOC_SO` (Linux, full path to `librenderdoc.so*`)
- `RENDERDOG_RENDERDOC_DIR` (install root, shared with `renderdog-automation`)

## Windows linking note

On Windows, the RenderDoc replay headers use `__declspec(dllimport)` for some API helpers. For this
experimental crate, enabling `cxx-replay` requires an import library:

- Set `RENDERDOG_REPLAY_RENDERDOC_LIB_DIR` (or `RENDERDOG_RENDERDOC_DIR`) to a directory containing `renderdoc.lib`.
