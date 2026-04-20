# renderdog-replay

Experimental RenderDoc *replay-session* bindings via a small C++ shim and the `cxx` crate.

This crate is **not published** to crates.io (`publish = false`), and the API is expected to change.
It is intentionally a small, stateful session adapter around RenderDoc's C++ replay API.

If you want stable capture/export/replay workflows, prefer `renderdog-automation` and
`qrenderdoc --python` from the workspace instead of depending on this crate directly.

See the [workspace README](../../README.md) for the stable crates and the MCP workflow.

## Status

- Goal: open an `.rdc` capture and expose a few session-scoped replay operations
  (e.g. list textures, pick pixels, save textures).
- Approach: dynamically load the local RenderDoc library (`renderdoc.dll` / `librenderdoc.so`) and call replay APIs.

The primary Rust entrypoint is `ReplayRuntime`.

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

This workspace pins the `third-party/renderdoc` submodule to a specific RenderDoc version, and
`renderdog-replay` must be run against the same version at runtime.

Check the pinned workspace version with:

- `cargo run -p renderdog-automation --example diagnose_environment`
- or `git -C third-party/renderdoc describe --tags --always`

If your installed RenderDoc version does not match the workspace pin, switch the submodule to the
matching version and rebuild, or install the matching RenderDoc version locally. Otherwise the
process may crash due to C++ ABI/layout mismatches.

Prefer `ReplayRuntime::new(...)` when selecting or validating a RenderDoc installation. It
validates the version match up front, and then `ReplayRuntime::new_session()` creates lightweight
sessions against that already-selected process-global runtime.

`renderdog-replay` also treats the RenderDoc runtime as process-global: once one RenderDoc module
has been loaded, switching to a different installation in the same process is rejected explicitly.

If it cannot be found via the OS loader, set one of:

- `RENDERDOG_REPLAY_RENDERDOC_DLL` (Windows, full path to `renderdoc.dll`)
- `RENDERDOG_REPLAY_RENDERDOC_SO` (Linux, full path to `librenderdoc.so*`)
- `RENDERDOG_RENDERDOC_DIR` (install root, shared with `renderdog-automation`)

## Debugging

- `RENDERDOG_REPLAY_TRACE=1`: print high-level steps to stderr.
- `RENDERDOG_REPLAY_TRACE_ALLOC=1`: also trace array allocations/frees (very noisy).
