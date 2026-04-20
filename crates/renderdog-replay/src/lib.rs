//! Experimental stateful RenderDoc replay-session bindings via a thin C++ shim.
//!
//! This crate intentionally stays small and low-level. It exposes a minimal
//! session-oriented API over RenderDoc's C++ replay interfaces for experiments.
//! Stable capture/export/replay workflows should prefer `renderdog-automation`.

mod ffi;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReplaySessionError {
    #[error("`renderdog-replay` requires the `cxx-replay` feature")]
    FeatureNotEnabled,

    #[error("failed to determine workspace RenderDoc replay header version")]
    WorkspaceVersionUnknown,

    #[error(
        "RenderDoc replay version mismatch: workspace headers expect `{expected}`, runtime library reports `{actual}`"
    )]
    VersionMismatch { expected: String, actual: String },

    #[error("pick_pixel returned {0} values (expected 4)")]
    InvalidPickPixelReturnLen(usize),

    #[cfg(feature = "cxx-replay")]
    #[error(transparent)]
    Cxx(#[from] cxx::Exception),
}

#[cfg(feature = "cxx-replay")]
pub struct ReplaySession {
    inner: cxx::UniquePtr<ffi::cxx_ffi::ReplaySession>,
}

#[cfg(feature = "cxx-replay")]
impl ReplaySession {
    pub fn new(renderdoc_path: Option<&str>) -> Result<Self, ReplaySessionError> {
        let path = renderdoc_path.unwrap_or("");
        let inner = ffi::cxx_ffi::replay_session_new(path)?;
        let runtime_version = inner.runtime_version_string()?;
        let expected_version =
            workspace_renderdoc_version().ok_or(ReplaySessionError::WorkspaceVersionUnknown)?;

        if !renderdoc_versions_match(&runtime_version, &expected_version) {
            return Err(ReplaySessionError::VersionMismatch {
                expected: expected_version,
                actual: runtime_version,
            });
        }

        Ok(Self { inner })
    }

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
pub struct ReplaySession;

#[cfg(not(feature = "cxx-replay"))]
impl ReplaySession {
    pub fn new(_renderdoc_path: Option<&str>) -> Result<Self, ReplaySessionError> {
        Err(ReplaySessionError::FeatureNotEnabled)
    }
}

#[cfg(feature = "cxx-replay")]
fn workspace_renderdoc_version() -> Option<String> {
    parse_workspace_renderdoc_version(include_str!(
        "../../../third-party/renderdoc/renderdoc/api/replay/version.h"
    ))
}

#[cfg(feature = "cxx-replay")]
fn parse_workspace_renderdoc_version(content: &str) -> Option<String> {
    let mut major: Option<String> = None;
    let mut minor: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("#define RENDERDOC_VERSION_MAJOR") {
            major = Some(value.trim().to_string());
        } else if let Some(value) = trimmed.strip_prefix("#define RENDERDOC_VERSION_MINOR") {
            minor = Some(value.trim().to_string());
        }
    }

    match (major, minor) {
        (Some(major), Some(minor)) => Some(format!("{major}.{minor}")),
        _ => None,
    }
}

#[cfg(feature = "cxx-replay")]
fn renderdoc_versions_match(runtime: &str, expected: &str) -> bool {
    match (
        normalize_renderdoc_version(runtime),
        normalize_renderdoc_version(expected),
    ) {
        (Some(runtime), Some(expected)) => runtime == expected,
        _ => false,
    }
}

#[cfg(feature = "cxx-replay")]
fn normalize_renderdoc_version(value: &str) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();

    for ch in value.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
            continue;
        }

        if !current.is_empty() {
            parts.push(std::mem::take(&mut current));
            if parts.len() == 2 {
                break;
            }
        }
    }

    if !current.is_empty() && parts.len() < 2 {
        parts.push(current);
    }

    if parts.len() >= 2 {
        Some(format!("{}.{}", parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(all(test, feature = "cxx-replay"))]
mod tests {
    use super::{
        normalize_renderdoc_version, parse_workspace_renderdoc_version, renderdoc_versions_match,
    };

    #[test]
    fn normalize_renderdoc_version_extracts_major_minor() {
        assert_eq!(
            normalize_renderdoc_version("v1.43"),
            Some("1.43".to_string())
        );
        assert_eq!(
            normalize_renderdoc_version("RenderDoc 1.43 stable"),
            Some("1.43".to_string())
        );
        assert_eq!(normalize_renderdoc_version("unknown"), None);
    }

    #[test]
    fn parse_workspace_renderdoc_version_reads_header_macros() {
        let content = r#"
#define RENDERDOC_VERSION_MAJOR 1
#define RENDERDOC_VERSION_MINOR 43
"#;

        assert_eq!(
            parse_workspace_renderdoc_version(content),
            Some("1.43".to_string())
        );
    }

    #[test]
    fn renderdoc_versions_match_uses_normalized_major_minor() {
        assert!(renderdoc_versions_match("v1.43", "1.43"));
        assert!(!renderdoc_versions_match("v1.42", "1.43"));
        assert!(!renderdoc_versions_match("custom-build", "1.43"));
    }
}
