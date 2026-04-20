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
//!
//! Low-level command execution and qrenderdoc scripting helpers are intentionally kept out of the
//! public crate surface. Most consumers should use `RenderDocInstallation` plus the replay/workflow
//! request/response types exported here.

mod command;
mod diagnostics;
mod normalize;
mod renderdoccmd;
mod replay;
mod scripting;
mod toolchain;
mod ui;
mod workflows;

pub use command::ToolInvocationError;
pub(crate) use command::{CommandSpec, run_command_expect_success, run_command_output_text};
pub use diagnostics::*;
pub(crate) use normalize::{normalize_capture_path, prepare_export_target};
pub use renderdoccmd::*;
pub use replay::*;
pub use scripting::QRenderDocExecutionError;
pub use toolchain::{DetectInstallationError, RenderDocInstallation, default_artifacts_dir};
pub(crate) use toolchain::{
    default_capture_basename, default_exports_dir, default_scripts_dir,
    resolve_export_output_dir_from_cwd, resolve_path_from_cwd, resolve_path_string_from_cwd,
};
pub use ui::*;
pub use workflows::*;
