# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

- None.

## [0.3.0] - 2026-05-03

### Breaking Changes

- RenderDoc: Updated the bundled API snapshot to RenderDoc 1.44. Replay sessions and diagnostics now expect a matching RenderDoc 1.44 runtime and report version mismatches explicitly.
- `renderdog-sys`: If you regenerate bindings yourself, provide the workspace RenderDoc submodule or `RENDERDOG_SYS_HEADER`; the published crate now relies on pregenerated bindings instead of shipping a fallback header.
- Automation/MCP: Simplified capture/export/find/replay request and response JSON into flatter workflow-oriented payloads. If you construct MCP requests by hand or deserialize tool output, review the updated examples before upgrading.
- In-app: Removed the deprecated `RenderDog` alias. Use `RenderDocInApp`.
- Automation/MCP: Removed obsolete low-level capture and command entrypoints. Use the bundle, replay, and diagnostics workflows instead.

### Added

- In-app: Richer object and command annotation helpers when RenderDoc API 1.7.0 is available.
- Linux: In-app helpers that can connect to an already-loaded RenderDoc library without loading a new one first.
- Replay: Experimental `renderdog-replay` crate for stateful replay sessions, currently covering texture listing, pixel picking, and texture PNG export.
- Diagnostics: Structured RenderDoc environment reports in `renderdog-automation` and `renderdog-mcp`, including installation discovery, `renderdoccmd` version probing, Vulkan layer analysis, warnings, and suggested fix commands.

### Changed

- Automation: `RenderDocInstallation` is now the main entrypoint for typed bundle and replay workflows.
- Capture/export: Actions and bindings JSONL are now handled as a single bundle workflow, with optional thumbnail saving and opening the capture in the UI.
- Replay: Output export now uses explicit event selection (`last_drawcall` or `event:<id>`), with `last_drawcall` as the default.
- Replay: Texture and output metadata now use typed, flatter fields that are easier to compare across Rust APIs and MCP tool results.
- Paths: Relative capture and output paths are normalized against the caller's working directory across automation and MCP workflows.

### Fixed

- qrenderdoc: Fixed headless `qrenderdoc --python` workflows with RenderDoc's embedded Python runtime.
- Windows: Fixed replay output serialization so normalized paths do not mix slash styles in generated requests.
- Diagnostics: Fixed RenderDoc version parsing for CLI strings such as `renderdoccmd x64 v1.44 ...`.
- Bindings: Fixed pregenerated binding checks across platforms where bindgen may choose different C integer aliases for RenderDoc enum wrappers.
- CI: Fixed clippy, binding regeneration, and workspace crate publish-order checks.

## [0.2.0] - 2026-01-10

### Added

- Export a searchable bindings index (`*.bindings.jsonl`) via `qrenderdoc --python` for fast offline querying.
- Headless replay helpers (via `qrenderdoc --python`): list textures, pick pixels, and save textures to PNG.
- MCP replay tools: `renderdoc_replay_list_textures`, `renderdoc_replay_pick_pixel`, `renderdoc_replay_save_texture_png`.
- MCP replay tool: `renderdoc_replay_save_outputs_png` (save current pipeline outputs to PNG).
- MCP utility tool: `renderdoc_find_events` (find matching `event_id`/marker paths for later replay).
- MCP one-shot tool: `renderdoc_find_events_and_save_outputs_png` (find event -> save outputs PNG).
- MCP one-shot bundle tool: `renderdoc_capture_and_export_bundle_jsonl` (capture + export actions + export bindings index).
- MCP export bundle tool: `renderdoc_export_bundle_jsonl` (actions + bindings index from an existing .rdc).
- Bundle tools can optionally save a thumbnail and/or open the capture in qrenderdoc UI.
- A practical RenderDoc playbook for validating clip-mask mapping: `docs/playbooks/fret-clip-mask.md`.
- A short guide for adding stable GPU pass markers: `docs/guides/gpu-markers.md`.
- A recommended adoption workflow section in the workspace README (capture -> markers -> UI inspection -> automation exports).
- A README section describing integration patterns with and without MCP.
- A README section documenting MCP client setup for Claude Code, Codex, and Gemini CLI.
- A README note recommending setting the MCP server working directory (`cwd`) to the project root.
- A `.gitattributes` file to stabilize line endings across platforms.
- MCP tool requests support an optional `cwd` to control relative path resolution per call.

### Fixed

- Make `qrenderdoc --python` scripts deterministic and non-interactive by using request/response JSON files and exiting cleanly.
- Resolve relative `capture_path`/`output_dir`/`output_path` against the caller working directory (so outputs don't end up under the internal run dir).
- Fix `renderdoc_replay_save_outputs_png` on Vulkan by handling output targets exposed as `renderdoc.Descriptor` objects.
- Make headless replay output exports choose a drawcall event by default (instead of `Present`), so outputs are usually non-empty.
- When using MCP tools with per-call `cwd`, resolve relative `executable`/`working_dir` for capture launch against that base directory.
- Externalize embedded `qrenderdoc --python` scripts into `crates/renderdog-automation/scripts/` for easier auditing and iteration.

## [0.1.0] - 2026-01-09

### Added

- `renderdog`: RenderDoc in-application API wrapper with runtime API version negotiation (1.6.0 down to 1.0.0).
- `renderdog-sys`: pregenerated low-level FFI bindings, with optional `bindgen` regeneration.
- `renderdog-automation`: out-of-process automation helpers for `renderdoccmd` and `qrenderdoc --python`.
- `renderdog-mcp`: an MCP server exposing capture/export/diagnostics workflows for AI agents.
- `renderdog-winit`: optional winit helpers (key mapping + window handle helpers).
- Vulkan layer diagnostics and environment hints (including `platform`/`arch` warnings).
