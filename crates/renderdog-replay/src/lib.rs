//! Experimental stateful RenderDoc replay-session bindings via a thin C++ shim.
//!
//! This crate intentionally stays small and low-level. It exposes a minimal
//! session-oriented API over RenderDoc's C++ replay interfaces for experiments.
//! Stable capture/export/replay workflows should prefer `renderdog-automation`.

mod ffi;

#[cfg(feature = "cxx-replay")]
use renderdog_sys::{renderdoc_versions_match, workspace_renderdoc_replay_version};
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
        let runtime_version = ffi::cxx_ffi::replay_runtime_probe(path)?;
        let expected_version = workspace_renderdoc_replay_version()
            .ok_or(ReplayRuntimeError::WorkspaceVersionUnknown)?
            .to_string();

        if !renderdoc_versions_match(&runtime_version, &expected_version) {
            return Err(ReplayRuntimeError::VersionMismatch {
                expected: expected_version,
                actual: runtime_version,
            });
        }

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
