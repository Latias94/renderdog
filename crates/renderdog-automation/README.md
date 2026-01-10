# renderdog-automation

Out-of-process automation helpers for RenderDoc.

Repository: https://github.com/Latias94/renderdog

This crate drives:

- `renderdoccmd capture` (injection-based capture)
- `qrenderdoc --python` (replay/export)

See the [workspace README](../../README.md) for example commands and MCP workflows.

Playbooks (practical debugging checklists):

- Clip-mask mapping (fret): https://github.com/Latias94/renderdog/blob/main/docs/playbooks/fret-clip-mask.md

Examples:

- Export actions + bindings bundle from an existing capture: `cargo run -p renderdog-automation --example export_bundle_from_capture -- <capture.rdc> [out_dir] [basename]`
- Find event IDs by marker substring: `cargo run -p renderdog-automation --example find_events -- <capture.rdc> [marker_contains]`
- Save current pipeline outputs to PNG (headless replay): `cargo run -p renderdog-automation --example replay_save_outputs_png -- <capture.rdc> [event_id] [out_dir] [basename]`
