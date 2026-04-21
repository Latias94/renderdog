# renderdog-mcp

MCP server exposing RenderDoc automation workflows (stdio transport).

Repository: https://github.com/Latias94/renderdog

See the [workspace README](../../README.md) for setup and available tools.

Recommended entrypoint tool: `renderdoc_capture_and_export_bundle_jsonl`.
For an existing capture, use: `renderdoc_export_bundle_jsonl`.

Diagnostics tools:

- `renderdoc_detect_installation`: resolve the RenderDoc install root and executables, then probe `renderdoccmd` version, workspace replay version compatibility, and Vulkan layer status.
- `renderdoc_diagnose_environment`: extend installation detection with platform, arch, elevation, discovered Vulkan layer manifests, Vulkan-related env vars, warnings, and suggested commands.
- `renderdoc_vulkanlayer_diagnose`: return parsed `renderdoccmd vulkanlayer --explain` output plus suggested fix commands.

Playbooks (practical debugging checklists):

- Clip-mask mapping (fret): https://github.com/Latias94/renderdog/blob/main/docs/playbooks/fret-clip-mask.md
