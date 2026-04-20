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

## RenderDoc version requirement (IMPORTANT)

`renderdog-replay` uses RenderDoc's **C++ replay API**. The version of the headers used at build
time must match the version of the RenderDoc library loaded at runtime.

This workspace pins the `third-party/renderdoc` submodule to **RenderDoc v1.43**, so you should run
`renderdog-replay` with **RenderDoc v1.43** (check with `renderdoccmd version`).

If you have a different version installed (e.g. v1.41), switch the submodule to the matching
version and rebuild. Otherwise the process may crash due to C++ ABI/layout mismatches.

If it cannot be found via the OS loader, set one of:

- `RENDERDOG_REPLAY_RENDERDOC_DLL` (Windows, full path to `renderdoc.dll`)
- `RENDERDOG_REPLAY_RENDERDOC_SO` (Linux, full path to `librenderdoc.so*`)
- `RENDERDOG_RENDERDOC_DIR` (install root, shared with `renderdog-automation`)

## Debugging

- `RENDERDOG_REPLAY_TRACE=1`: print high-level steps to stderr.
- `RENDERDOG_REPLAY_TRACE_ALLOC=1`: also trace array allocations/frees (very noisy).
