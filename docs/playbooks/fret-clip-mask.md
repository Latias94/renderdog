# Playbook: Verify Fret Clip Mask Mapping (RenderDoc)

This playbook documents a *contract checklist* to validate clip-mask generation and sampling logic
using a RenderDoc capture (e.g. `fret_capture*.rdc`).

The goal is to quickly answer: “Does the mask coordinate mapping match the panel/effect scissor and
the clip-stack transform?”

## Prerequisites

- A RenderDoc capture file: `fret_capture*.rdc`
- `qrenderdoc` (UI) for interactive inspection
- Optional: `renderdog-automation` / `renderdog-mcp` for quick exports and pixel/texture checks

## Checklist (UI)

### 1) Locate key passes (Event Browser)

Search by marker/label in Event Browser:

- `fret clip mask pass`
- `fret upscale-nearest mask pass`
- `fret blur mask pass`
- `fret color-adjust ...`
- `fret composite premul masked pass` (the `FilterContent` path that uses the clip-stack masked pipeline)

### 2) Verify ClipMask render target size

In `fret clip mask pass`:

- The output RT size should match the corresponding `Mask{0/1/2}` effect `viewport_rect` after
  scaling (not the full window size).
- Confirm the bound viewport/scissor corresponds to the panel/effect rect.

### 3) Verify ClipMask coordinate mapping (uniforms)

Inspect the relevant uniform data (e.g. `ViewportUniform`):

- `mask_viewport_origin` / `mask_viewport_size` must match the effect scissor rect (panel position + size).
- In shader mapping, the sampling should behave like:
  - `clip_alpha(mask_viewport_origin + (pos + 0.5) * scale)`
  - and should not use the full-window `viewport_size`.

### 4) Verify mask sampling coordinates (most important)

In any `... mask pass`:

- In the fragment shader, `local_x/local_y` should be derived from `pixel - mask_viewport_origin`.
- Out-of-bounds should return `0` immediately (do not remap the whole window into a small mask).
- Check the bound `mask_texture` size vs `mask_viewport_size` matches expectations (tier scaling).

### 5) Verify `FilterContent` composite semantics

In `fret composite premul masked pass`:

- There should be no `mask_texture` bound (mask should be `None`).
- The clip behavior should come from `mask_uniform_index` selecting the clip-stack masked pipeline:
  - bounds are for computing geometry bounds, not performing clip sampling directly.

### 6) Verify clip-stack matrix data (SSBO)

In composite / masked passes:

- Inspect clip stack SSBO entries.
- `ClipRRectUniform.inv0/inv1` must match the panel rrect transform you expect.
  This is the core data source for “matrix correctness”.

## Optional quick checks (automation)

These checks don’t replace UI inspection, but help validate outputs quickly:

- Export actions/bindings for grep-friendly search
  - `renderdog-automation`: `export_actions_from_capture` (example)
  - `renderdog-mcp`: `renderdoc_export_actions_jsonl` / `renderdoc_export_bindings_index_jsonl`
- Export textures / pick pixels (headless replay via `qrenderdoc --python`)
  - `renderdog-automation`: `replay_list_textures`, `replay_pick_pixel`, `replay_save_texture_png`
  - `renderdog-mcp`: `renderdoc_replay_list_textures`, `renderdoc_replay_pick_pixel`,
    `renderdoc_replay_save_texture_png`

