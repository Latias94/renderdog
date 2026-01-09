# renderdog

[![CI](https://github.com/Latias94/renderdog/actions/workflows/ci.yml/badge.svg)](https://github.com/Latias94/renderdog/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/renderdog.svg)](https://crates.io/crates/renderdog)
[![docs.rs](https://docs.rs/renderdog/badge.svg)](https://docs.rs/renderdog)

RenderDoc in-application API wrapper + automation helpers + an MCP server.

This project is **not affiliated with RenderDoc**. RenderDoc itself is developed by Baldur Karlsson:
https://github.com/baldurk/renderdoc

Repository: https://github.com/Latias94/renderdog

## Goals

- Provide a small, safe-ish Rust wrapper for RenderDoc's in-app capture API.
- Provide out-of-process automation that is usable by an AI agent (capture + export searchable artifacts).
- Keep the "replay/analysis" side in `qrenderdoc --python` for fast iteration (no Rust replay bindings).

## Crates

- `renderdog`: in-app API wrapper (connect to injected RenderDoc or dynamically load it).
- `renderdog-sys`: low-level FFI bindings (pregenerated, optional bindgen regeneration).
- `renderdog-automation`: out-of-process automation helpers (`renderdoccmd`, `qrenderdoc --python` workflows).
- `renderdog-mcp`: MCP server exposing automation workflows.
- `renderdog-winit`: optional `winit` helpers (key mapping + window-handle helpers).

## Platform support

- Windows: primary target for in-app capture and automation.
- Linux (x86_64): supported; in-app loader uses `librenderdoc.so`/`librenderdoc.so.1` by default.
- macOS: RenderDoc is experimental and not officially supported for debugging; `renderdog` may compile but capture/replay can be unreliable.

## Installation (crates.io)

- Library: `cargo add renderdog`
- MCP server (binary): `cargo install renderdog-mcp`

## Prerequisites

You need a local RenderDoc installation that provides both:

- `renderdoccmd` (used for injection-based capture)
- `qrenderdoc` (used for replay/analysis/export via `qrenderdoc --python`)

`renderdog-automation` / `renderdog-mcp` tries to detect RenderDoc via:

- `RENDERDOG_RENDERDOC_DIR` (preferred: points to the RenderDoc install root)
- Windows default: `C:\Program Files\RenderDoc`
- `PATH` (if `renderdoccmd` / `qrenderdoc` are discoverable)

## In-app usage

Run the example:

`cargo run -p renderdog --example in_app_capture`

Key points:

- Loader tries API versions from `1.6.0` down to `1.0.0` via `RENDERDOC_GetAPI`.
- Windows injected connect uses `GetModuleHandleA("renderdoc.dll")` and does not call `LoadLibrary`.
- Explicit load is available via `RenderDog::load("renderdoc.dll")` / `RenderDocInApp::try_load_and_connect(...)`.
- Linux optional: connect only if already loaded (RTLD_NOLOAD): `RenderDocInApp::try_connect_noload_default()` or `RenderDog::new_noload_first()`.
- Thread-safety: in-app handles are `Send` but `!Sync` and not `Clone`. For cross-thread usage, wrap in `Arc<Mutex<...>>` to serialize calls.

## Examples

- In-app connect (injected-only, Windows): `cargo run -p renderdog --example in_app_injected_only`
- In-app options/overlay/output template: `cargo run -p renderdog --example in_app_options_overlay`
- Automation one-shot capture + export: `cargo run -p renderdog-automation --example one_shot_capture_export -- <exe> [args...]`
- Winit hotkey capture (F12): `cargo run -p renderdog-winit --example winit_hotkey_capture`

## MCP workflow (one-shot)

Run the server locally (stdio transport):

- From crates.io: `renderdog-mcp`
- From source: `cargo run -p renderdog-mcp`

`renderdog-mcp` provides a one-shot tool that can:

1) launch a target with `renderdoccmd capture`
2) trigger capture via target-control
3) export searchable artifacts (`.actions.jsonl` + `.summary.json`)

The export supports optional filters:

- `only_drawcalls`, `marker_prefix`
- `event_id_min/event_id_max`
- `name_contains`, `marker_contains` (+ `case_sensitive`)

## Logging

`renderdog-mcp` uses `tracing` and honors `RUST_LOG`:

- Default: `info`
- Debug (includes detailed command failure context): `RUST_LOG=debug renderdog-mcp`

## Vulkan troubleshooting

If Vulkan capture doesn't work, RenderDoc's Vulkan layer registration may be missing or conflicting.
You can diagnose it via:

- MCP tool: `renderdoc_vulkanlayer_diagnose`
- MCP tool: `renderdoc_diagnose_environment` (includes env var hints and `platform`/`arch`)
- CLI: `"renderdoccmd" vulkanlayer --explain`

If it needs attention, suggested fixes typically include:

- `"renderdoccmd" vulkanlayer --register --user`
- `"renderdoccmd" vulkanlayer --register --system` (administrator)

## Troubleshooting

- RenderDoc not detected: set `RENDERDOG_RENDERDOC_DIR` to the RenderDoc install root.
- Linux in-app load fails: ensure `librenderdoc.so` is available on the loader search path
  (e.g. install RenderDoc system-wide, or set `LD_LIBRARY_PATH` appropriately).
- Vulkan capture fails: use `renderdoc_diagnose_environment` / `renderdoc_vulkanlayer_diagnose` and follow suggested fixes.

## Optional: RenderDoc submodule (for bindings regeneration)

`renderdog-sys` ships with pregenerated bindings by default. If you want to regenerate bindings via
`bindgen`, you can add RenderDoc as a git submodule at `third-party/renderdoc` and run:

`RENDERDOG_SYS_REGEN_BINDINGS=1 cargo build -p renderdog-sys --features bindgen`

You can also point to a specific header via `RENDERDOG_SYS_HEADER=/path/to/renderdoc_app.h`.

## Author

Maintained by `Latias94`.

## License

MIT. See `LICENSE-MIT`.
