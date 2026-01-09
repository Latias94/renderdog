# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [Unreleased]

## [0.1.0] - 2026-01-09

### Added

- `renderdog`: RenderDoc in-application API wrapper with runtime API version negotiation (1.6.0 down to 1.0.0).
- `renderdog-sys`: pregenerated low-level FFI bindings, with optional `bindgen` regeneration.
- `renderdog-automation`: out-of-process automation helpers for `renderdoccmd` and `qrenderdoc --python`.
- `renderdog-mcp`: an MCP server exposing capture/export/diagnostics workflows for AI agents.
- `renderdog-winit`: optional winit helpers (key mapping + window handle helpers).
- Vulkan layer diagnostics and environment hints (including `platform`/`arch` warnings).
