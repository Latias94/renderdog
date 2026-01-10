//! Out-of-process automation helpers for RenderDoc.
//!
//! This crate drives RenderDoc tooling via external processes:
//! - `renderdoccmd capture` for injection-based capture
//! - `qrenderdoc --python` for replay/analysis/export (e.g. `.actions.jsonl`)
//!
//! Most failures are surfaced with detailed context (args/cwd/status/stdout/stderr) to make
//! debugging environment issues easier.
//!
//! To override the auto-detection of RenderDoc tools, set:
//! - `RENDERDOG_RENDERDOC_DIR=<RenderDoc install root>`

mod command;
mod diagnostics;
mod renderdoccmd;
mod replay;
mod scripting;
mod toolchain;
mod ui;
mod workflows;

pub use command::*;
pub use diagnostics::*;
pub use renderdoccmd::*;
pub use replay::*;
pub use scripting::*;
pub use toolchain::*;
pub use ui::*;
pub use workflows::*;
