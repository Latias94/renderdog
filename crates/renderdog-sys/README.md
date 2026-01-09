# renderdog-sys

Low-level FFI bindings for RenderDoc's in-application API (`renderdoc_app.h`).

Repository: https://github.com/Latias94/renderdog

This crate ships pregenerated bindings for docs.rs. Maintainers can regenerate bindings via:

- `RENDERDOG_SYS_REGEN_BINDINGS=1 cargo build -p renderdog-sys --features bindgen`
- `python scripts/regen_bindings.py`

See the [workspace README](../../README.md) for overall project goals and setup.
