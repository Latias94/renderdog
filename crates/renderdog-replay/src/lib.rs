//! Experimental stateful RenderDoc replay-session bindings via a thin C++ shim.
//!
//! This crate intentionally stays small and low-level. It exposes a minimal
//! session-oriented API over RenderDoc's C++ replay interfaces for experiments.
//! Stable capture/export/replay workflows should prefer `renderdog-automation`.

mod ffi;

use renderdog_sys::renderdoc_versions_match;
#[cfg(feature = "cxx-replay")]
use renderdog_sys::workspace_renderdoc_replay_version;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplayRuntimeError {
    #[error("`renderdog-replay` requires the `cxx-replay` feature")]
    FeatureNotEnabled,

    #[error("failed to determine workspace RenderDoc replay header version")]
    WorkspaceVersionUnknown,

    #[error(
        "RenderDoc replay version mismatch: workspace headers expect `{expected}`, runtime library reports `{actual}`"
    )]
    VersionMismatch { expected: String, actual: String },

    #[cfg(feature = "cxx-replay")]
    #[error(transparent)]
    Cxx(#[from] cxx::Exception),
}

#[derive(Debug, Error)]
pub enum ReplaySessionError {
    #[error("`renderdog-replay` requires the `cxx-replay` feature")]
    FeatureNotEnabled,

    #[error("pick_pixel returned {0} values (expected 4)")]
    InvalidPickPixelReturnLen(usize),

    #[cfg(feature = "cxx-replay")]
    #[error(transparent)]
    Cxx(#[from] cxx::Exception),
}

fn validate_runtime_version(
    runtime_version: String,
    workspace_version: Option<&str>,
) -> Result<String, ReplayRuntimeError> {
    let expected_version = workspace_version.ok_or(ReplayRuntimeError::WorkspaceVersionUnknown)?;

    if !renderdoc_versions_match(&runtime_version, expected_version) {
        return Err(ReplayRuntimeError::VersionMismatch {
            expected: expected_version.to_string(),
            actual: runtime_version,
        });
    }

    Ok(runtime_version)
}

#[cfg(feature = "cxx-replay")]
pub struct ReplayRuntime {
    runtime_version: String,
}

#[cfg(feature = "cxx-replay")]
pub struct ReplaySession {
    inner: cxx::UniquePtr<ffi::cxx_ffi::ReplaySession>,
}

#[cfg(feature = "cxx-replay")]
impl ReplayRuntime {
    pub fn new(renderdoc_path: Option<&str>) -> Result<Self, ReplayRuntimeError> {
        let path = renderdoc_path.unwrap_or("");
        let runtime_version = validate_runtime_version(
            ffi::cxx_ffi::replay_runtime_probe(path)?,
            workspace_renderdoc_replay_version(),
        )?;

        Ok(Self { runtime_version })
    }

    pub fn runtime_version(&self) -> &str {
        &self.runtime_version
    }

    pub fn new_session(&self) -> Result<ReplaySession, ReplaySessionError> {
        let inner = ffi::cxx_ffi::replay_session_new_current()?;
        Ok(ReplaySession { inner })
    }
}

#[cfg(feature = "cxx-replay")]
impl ReplaySession {
    pub fn open_capture(&mut self, capture_path: &str) -> Result<(), ReplaySessionError> {
        let pinned = self.inner.pin_mut();
        pinned.open_capture(capture_path)?;
        Ok(())
    }

    pub fn set_frame_event(&mut self, event_id: u32) -> Result<(), ReplaySessionError> {
        let pinned = self.inner.pin_mut();
        pinned.set_frame_event(event_id)?;
        Ok(())
    }

    pub fn list_textures_json(&self) -> Result<String, ReplaySessionError> {
        Ok(self.inner.list_textures_json()?)
    }

    pub fn pick_pixel(
        &self,
        texture_index: u32,
        x: u32,
        y: u32,
    ) -> Result<[f32; 4], ReplaySessionError> {
        let v = self.inner.pick_pixel(texture_index, x, y)?;
        if v.len() != 4 {
            return Err(ReplaySessionError::InvalidPickPixelReturnLen(v.len()));
        }
        Ok([v[0], v[1], v[2], v[3]])
    }

    pub fn save_texture_png(
        &self,
        texture_index: u32,
        output_path: &str,
    ) -> Result<(), ReplaySessionError> {
        self.inner.save_texture_png(texture_index, output_path)?;
        Ok(())
    }
}

#[cfg(not(feature = "cxx-replay"))]
pub struct ReplayRuntime;

#[cfg(not(feature = "cxx-replay"))]
pub struct ReplaySession;

#[cfg(not(feature = "cxx-replay"))]
impl ReplayRuntime {
    pub fn new(_renderdoc_path: Option<&str>) -> Result<Self, ReplayRuntimeError> {
        Err(ReplayRuntimeError::FeatureNotEnabled)
    }

    pub fn new_session(&self) -> Result<ReplaySession, ReplaySessionError> {
        Err(ReplaySessionError::FeatureNotEnabled)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReplayRuntimeError, validate_runtime_version};

    fn workspace_replay_version() -> &'static str {
        renderdog_sys::workspace_renderdoc_replay_version()
            .expect("workspace replay version should be available in tests")
    }

    fn mismatched_replay_version() -> &'static str {
        match workspace_replay_version() {
            "0.0" => "999.999",
            _ => "0.0",
        }
    }

    #[test]
    fn validate_runtime_version_accepts_normalized_match() {
        let workspace_version = workspace_replay_version();
        let runtime_label = format!("RenderDoc v{workspace_version} loaded");
        let runtime_version =
            validate_runtime_version(runtime_label.clone(), Some(workspace_version))
                .expect("version should match");

        assert_eq!(runtime_version, runtime_label);
    }

    #[test]
    fn validate_runtime_version_rejects_mismatch() {
        let workspace_version = workspace_replay_version();
        let mismatched_version = mismatched_replay_version();
        let err = validate_runtime_version(mismatched_version.to_string(), Some(workspace_version))
            .expect_err("version mismatch should fail fast");

        match err {
            ReplayRuntimeError::VersionMismatch { expected, actual } => {
                assert_eq!(expected, workspace_version);
                assert_eq!(actual, mismatched_version);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn validate_runtime_version_requires_workspace_version() {
        let err = validate_runtime_version(workspace_replay_version().to_string(), None)
            .expect_err("missing workspace version should fail");

        assert!(matches!(err, ReplayRuntimeError::WorkspaceVersionUnknown));
    }
}
