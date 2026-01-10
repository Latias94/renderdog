# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

### Added

- (TBD)

## [0.2.0] - 2026-01-10

### Added

- Export a searchable bindings index (`*.bindings.jsonl`) via `qrenderdoc --python` for fast offline querying.
- Headless replay helpers (via `qrenderdoc --python`): list textures, pick pixels, and save textures to PNG.
- MCP replay tools: `renderdoc_replay_list_textures`, `renderdoc_replay_pick_pixel`, `renderdoc_replay_save_texture_png`.
- MCP replay tool: `renderdoc_replay_save_outputs_png` (save current pipeline outputs to PNG).
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

### Fixed

- Make `qrenderdoc --python` scripts deterministic and non-interactive by using request/response JSON files and exiting cleanly.
- Resolve relative `capture_path`/`output_dir`/`output_path` against the caller working directory (so outputs don't end up under the internal run dir).
- Fix `renderdoc_replay_save_outputs_png` on Vulkan by handling output targets exposed as `renderdoc.Descriptor` objects.

## [0.1.0] - 2026-01-09

### Added

- `renderdog`: RenderDoc in-application API wrapper with runtime API version negotiation (1.6.0 down to 1.0.0).
- `renderdog-sys`: pregenerated low-level FFI bindings, with optional `bindgen` regeneration.
- `renderdog-automation`: out-of-process automation helpers for `renderdoccmd` and `qrenderdoc --python`.
- `renderdog-mcp`: an MCP server exposing capture/export/diagnostics workflows for AI agents.
- `renderdog-winit`: optional winit helpers (key mapping + window handle helpers).
- Vulkan layer diagnostics and environment hints (including `platform`/`arch` warnings).
