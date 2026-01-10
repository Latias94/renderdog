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
- Keep the stable "replay/analysis" workflow in `qrenderdoc --python` for fast iteration.
- Optionally experiment with a minimal Rust-friendly replay shim (`renderdog-replay`, not published).

## Crates

- `renderdog`: in-app API wrapper (connect to injected RenderDoc or dynamically load it).
- `renderdog-sys`: low-level FFI bindings (pregenerated, optional bindgen regeneration).
- `renderdog-automation`: out-of-process automation helpers (`renderdoccmd`, `qrenderdoc --python` workflows).
- `renderdog-mcp`: MCP server exposing automation workflows.
- `renderdog-winit`: optional `winit` helpers (key mapping + window-handle helpers).
- `renderdog-replay`: experimental replay shim (C++/cxx, not published to crates.io).

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

## Integration patterns (with or without MCP)

RenderDoc analysis depends heavily on having stable pass markers/labels. Regardless of which
workflow you choose below, make sure your renderer emits consistent GPU markers for each pass
so you can search them in RenderDoc's Event Browser.

### Without MCP

Use this when you want a local/manual workflow (or your own automation) without an MCP client.

- In-app capture control (integrate `renderdog` into your renderer):
  - `cargo add renderdog`
  - Typical flow:
    - connect/load RenderDoc (`RenderDog::new()` / `RenderDocInApp::try_connect_or_load_default()`)
    - optionally set capture path template (`set_capture_file_path_template...`)
    - trigger capture (`trigger_capture` or `start_frame_capture`/`end_frame_capture`)
- Out-of-process automation from CLI (no MCP):
  - Capture + export: `cargo run -p renderdog-automation --example one_shot_capture_export -- <exe> [args...]`
  - Export from existing `.rdc`: `cargo run -p renderdog-automation --example export_bundle_from_capture -- <capture.rdc> [out_dir] [basename]`
  - Headless replay outputs: `cargo run -p renderdog-automation --example replay_save_outputs_png -- <capture.rdc> [event_id] [out_dir] [basename]`

### With MCP (AI-friendly)

Use this when you want an AI agent to drive capture/replay/export via tool calls.

- Run the server (stdio): `cargo run -p renderdog-mcp` (or `cargo install renderdog-mcp` then `renderdog-mcp`)
- Recommended tool entrypoints:
  - One-shot capture + export bundle: `renderdoc_capture_and_export_bundle_jsonl`
  - Export bundle from an existing `.rdc`: `renderdoc_export_bundle_jsonl`
  - Headless replay outputs: `renderdoc_replay_save_outputs_png`

## Examples

- In-app connect (injected-only, Windows): `cargo run -p renderdog --example in_app_injected_only`
- In-app options/overlay/output template: `cargo run -p renderdog --example in_app_options_overlay`
- Automation one-shot capture + export: `cargo run -p renderdog-automation --example one_shot_capture_export -- <exe> [args...]`
- Automation export bundle from capture: `cargo run -p renderdog-automation --example export_bundle_from_capture -- <capture.rdc> [out_dir] [basename]`
- Automation save pipeline outputs to PNG: `cargo run -p renderdog-automation --example replay_save_outputs_png -- <capture.rdc> [event_id] [out_dir] [basename]`
- Winit hotkey capture (F12): `cargo run -p renderdog-winit --example winit_hotkey_capture`

## MCP workflow (one-shot)

Run the server locally (stdio transport):

- From crates.io: `renderdog-mcp`
- From source: `cargo run -p renderdog-mcp`

`renderdog-mcp` provides one-shot tools that can:

1) launch a target with `renderdoccmd capture`
2) trigger capture via target-control
3) export searchable artifacts

Recommended: `renderdoc_capture_and_export_bundle_jsonl` (exports both actions + bindings index).
For an existing capture, use: `renderdoc_export_bundle_jsonl`.
Bundle tools also support optional `save_thumbnail` / `open_capture_ui` helpers.

Artifacts:

- actions tree: `.actions.jsonl` + `.summary.json`
- bindings index: `.bindings.jsonl` + `.bindings_summary.json` (shader names + resource bindings per drawcall)

The export supports optional filters:

- `only_drawcalls`, `marker_prefix`
- `event_id_min/event_id_max`
- `name_contains`, `marker_contains` (+ `case_sensitive`)

## Debug playbooks

Practical checklists for validating real-world rendering issues:

- Clip-mask mapping (fret): `docs/playbooks/fret-clip-mask.md`

## Adopting in your renderer (recommended workflow)

If you want an AI agent (or a human) to debug a renderer effectively with RenderDoc, aim for a
workflow that is *repeatable and searchable*:

1) Capture a frame (`.rdc`)
   - In-app control: integrate `renderdog` and trigger captures programmatically (or via hotkeys).
   - Injection-based: use `renderdog-automation` / `renderdog-mcp` (wraps `renderdoccmd capture`).
2) Make passes easy to find
   - Emit stable GPU markers / debug labels for key passes (RenderDoc Event Browser search relies on these).
3) Inspect in `qrenderdoc` UI, then automate exports
   - Use the playbooks for step-by-step validation (e.g. `docs/playbooks/fret-clip-mask.md`).
   - Export actions/bindings to JSONL for grep-friendly queries.
   - Use headless replay helpers (`qrenderdoc --python`) to list textures / pick pixels / save PNGs.

## Headless replay helpers (qrenderdoc --python)

In addition to exporting actions/bindings, `renderdog-automation` and `renderdog-mcp` provide
headless replay helpers (implemented via `qrenderdoc --python`) that are useful for quick sanity
checks:

- List textures in a capture
- Pick a pixel from a texture
- Save a texture to PNG
- Save current pipeline outputs (RTs + optional depth) to PNG

These are exposed as:

- `renderdog-automation` examples:
  - `replay_list_textures`
  - `replay_pick_pixel`
  - `replay_save_texture_png`
  - `replay_save_outputs_png`
- `renderdog-mcp` tools:
  - `renderdoc_replay_list_textures`
  - `renderdoc_replay_pick_pixel`
  - `renderdoc_replay_save_texture_png`
  - `renderdoc_replay_save_outputs_png`

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
`bindgen`, you can use the optional submodule at `third-party/renderdoc` and run:

`RENDERDOG_SYS_REGEN_BINDINGS=1 cargo build -p renderdog-sys --features bindgen`

You can also point to a specific header via `RENDERDOG_SYS_HEADER=/path/to/renderdoc_app.h`.

Alternatively, use the helper script (maintainers):

- Update pregenerated bindings: `python scripts/regen_bindings.py`
- Check without writing: `python scripts/regen_bindings.py --check`

## Packaging note (workspace vs crates.io)

This repository is a Cargo workspace. When you add new cross-crate APIs locally (e.g. new
`renderdog-automation` functions used by `renderdog-mcp`), `cargo package -p renderdog-mcp` will
only pass *after* the corresponding `renderdog-automation` version is published to crates.io.

During development, use workspace builds/tests (`cargo build`, `cargo nextest run`). For packaging
checks, you can run `cargo package -p <crate> --allow-dirty` (but verification may still fail if it
depends on unpublished crates).

## License

Dual-licensed under `MIT OR Apache-2.0`. See `LICENSE-MIT` and `LICENSE-APACHE`.
