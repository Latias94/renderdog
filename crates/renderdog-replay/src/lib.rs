//! Experimental stateful RenderDoc replay-session bindings via a thin C++ shim.
//!
//! This crate intentionally stays small and low-level. It exposes a minimal
//! session-oriented API over RenderDoc's C++ replay interfaces for experiments.
//! Stable capture/export/replay workflows should prefer `renderdog-automation`.

mod ffi;
mod version_policy;

use serde::Deserialize;
use thiserror::Error;

#[cfg(any(feature = "cxx-replay", test))]
use crate::version_policy::{renderdoc_versions_match, workspace_renderdoc_replay_version};

#[derive(Debug, Error)]
pub enum ReplayRuntimeError {
    #[error("`renderdog-replay` requires the `cxx-replay` feature")]
    FeatureNotEnabled,

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

    #[error("failed to decode replay texture list: {0}")]
    InvalidTextureList(#[from] serde_json::Error),

    #[cfg(feature = "cxx-replay")]
    #[error(transparent)]
    Cxx(#[from] cxx::Exception),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReplayTextureInfo {
    pub index: u32,
    pub resource_id: u64,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mips: u32,
    pub array_size: u32,
    pub ms_samp: u32,
    pub byte_size: u64,
}

#[cfg(any(feature = "cxx-replay", test))]
fn validate_runtime_version(
    runtime_version: String,
    workspace_version: &str,
) -> Result<String, ReplayRuntimeError> {
    if !renderdoc_versions_match(&runtime_version, workspace_version) {
        return Err(ReplayRuntimeError::VersionMismatch {
            expected: workspace_version.to_string(),
            actual: runtime_version,
        });
    }

    Ok(runtime_version)
}

#[cfg(any(feature = "cxx-replay", test))]
fn parse_texture_list(serialized: &str) -> Result<Vec<ReplayTextureInfo>, ReplaySessionError> {
    serde_json::from_str(serialized).map_err(ReplaySessionError::InvalidTextureList)
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

    pub fn list_textures(&self) -> Result<Vec<ReplayTextureInfo>, ReplaySessionError> {
        parse_texture_list(&self.inner.list_textures_serialized()?)
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
    use crate::version_policy::workspace_renderdoc_replay_version;

    use super::{
        ReplayRuntimeError, ReplaySessionError, parse_texture_list, validate_runtime_version,
    };

    fn workspace_replay_version() -> &'static str {
        workspace_renderdoc_replay_version()
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
        let runtime_version = validate_runtime_version(runtime_label.clone(), workspace_version)
            .expect("version should match");

        assert_eq!(runtime_version, runtime_label);
    }

    #[test]
    fn validate_runtime_version_rejects_mismatch() {
        let workspace_version = workspace_replay_version();
        let mismatched_version = mismatched_replay_version();
        let err = validate_runtime_version(mismatched_version.to_string(), workspace_version)
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
    fn parse_texture_list_decodes_serialized_snapshot() {
        let textures = parse_texture_list(
            r#"
            [
              {
                "index": 3,
                "resource_id": 42,
                "name": "Color Target",
                "width": 1920,
                "height": 1080,
                "depth": 1,
                "mips": 1,
                "array_size": 1,
                "ms_samp": 4,
                "byte_size": 8294400
              }
            ]
            "#,
        )
        .expect("serialized texture list should decode");

        assert_eq!(textures.len(), 1);
        let texture = &textures[0];
        assert_eq!(texture.index, 3);
        assert_eq!(texture.resource_id, 42);
        assert_eq!(texture.name, "Color Target");
        assert_eq!(texture.width, 1920);
        assert_eq!(texture.height, 1080);
        assert_eq!(texture.depth, 1);
        assert_eq!(texture.mips, 1);
        assert_eq!(texture.array_size, 1);
        assert_eq!(texture.ms_samp, 4);
        assert_eq!(texture.byte_size, 8294400);
    }

    #[test]
    fn parse_texture_list_rejects_invalid_json() {
        let err = parse_texture_list("{").expect_err("invalid JSON should fail");
        assert!(matches!(err, ReplaySessionError::InvalidTextureList(_)));
    }
}
