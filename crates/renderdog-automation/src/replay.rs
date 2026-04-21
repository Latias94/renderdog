use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::qrenderdoc_jobs::{
    REPLAY_LIST_TEXTURES_JOB, REPLAY_PICK_PIXEL_JOB, REPLAY_SAVE_OUTPUTS_PNG_JOB,
    REPLAY_SAVE_TEXTURE_PNG_JOB,
};
use crate::scripting::PrepareQRenderDocJsonRequest;
use crate::{CaptureInput, ExportOutput, OutputFile, RenderDocInstallation};
use crate::{QRenderDocJsonError, normalize_capture_path};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
}

impl ReplayListTexturesRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: normalize_capture_path(cwd, &self.capture_path),
            ..self.clone()
        }
    }
}

impl PrepareQRenderDocJsonRequest for ReplayListTexturesRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.normalized_in_cwd(cwd))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayTextureInfo {
    pub index: u32,
    pub resource_id: u64,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mips: u32,
    pub arraysize: u32,
    pub ms_samp: u32,
    pub byte_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub textures: Vec<ReplayTextureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelRequest {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
}

impl ReplayPickPixelRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: normalize_capture_path(cwd, &self.capture_path),
            ..self.clone()
        }
    }
}

impl PrepareQRenderDocJsonRequest for ReplayPickPixelRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.normalized_in_cwd(cwd))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
    pub rgba: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveTexturePngRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    #[serde(flatten)]
    pub output: OutputFile,
}

impl ReplaySaveTexturePngRequest {
    pub(crate) fn resolved_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture: self.capture.normalized_in_cwd(cwd),
            output: self.output.resolved_in_cwd(cwd),
            ..self.clone()
        }
    }
}

impl PrepareQRenderDocJsonRequest for ReplaySaveTexturePngRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.resolved_in_cwd(cwd))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveTexturePngResponse {
    pub capture_path: String,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(default)]
    pub event_id: Option<u32>,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(default)]
    pub include_depth: bool,
}

impl ReplaySaveOutputsPngRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Result<Self, std::io::Error> {
        let (capture, output) = self.output.normalized_for_capture(cwd, &self.capture)?;

        Ok(Self {
            capture,
            output,
            ..self.clone()
        })
    }
}

impl PrepareQRenderDocJsonRequest for ReplaySaveOutputsPngRequest {
    type Error = ReplaySaveOutputsPngError;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error> {
        self.normalized_in_cwd(cwd)
            .map_err(ReplaySaveOutputsPngError::CreateOutputDir)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySavedImage {
    pub kind: String,
    pub index: Option<u32>,
    pub resource_id: u64,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngResponse {
    pub capture_path: String,
    pub event_id: u32,
    pub outputs: Vec<ReplaySavedImage>,
}

pub type ReplayListTexturesError = QRenderDocJsonError;
pub type ReplayPickPixelError = QRenderDocJsonError;
pub type ReplaySaveTexturePngError = QRenderDocJsonError;

#[derive(Debug, Error)]
pub enum ReplaySaveOutputsPngError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("replay job failed: {0}")]
    Job(#[from] QRenderDocJsonError),
}

impl RenderDocInstallation {
    pub fn replay_list_textures(
        &self,
        cwd: &Path,
        req: &ReplayListTexturesRequest,
    ) -> Result<ReplayListTexturesResponse, ReplayListTexturesError> {
        self.run_prepared_qrenderdoc_json_job(cwd, REPLAY_LIST_TEXTURES_JOB, req)
    }

    pub fn replay_pick_pixel(
        &self,
        cwd: &Path,
        req: &ReplayPickPixelRequest,
    ) -> Result<ReplayPickPixelResponse, ReplayPickPixelError> {
        self.run_prepared_qrenderdoc_json_job(cwd, REPLAY_PICK_PIXEL_JOB, req)
    }

    pub fn replay_save_texture_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveTexturePngRequest,
    ) -> Result<ReplaySaveTexturePngResponse, ReplaySaveTexturePngError> {
        self.run_prepared_qrenderdoc_json_job(cwd, REPLAY_SAVE_TEXTURE_PNG_JOB, req)
    }

    pub fn replay_save_outputs_png(
        &self,
        cwd: &Path,
        req: &ReplaySaveOutputsPngRequest,
    ) -> Result<ReplaySaveOutputsPngResponse, ReplaySaveOutputsPngError> {
        self.run_prepared_qrenderdoc_json_job(cwd, REPLAY_SAVE_OUTPUTS_PNG_JOB, req)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::Value;

    use super::{ReplaySaveOutputsPngRequest, ReplaySaveTexturePngRequest};
    use crate::{CaptureInput, ExportOutput, OutputFile};

    #[test]
    fn replay_save_texture_request_resolves_capture_and_output_paths() {
        let req = ReplaySaveTexturePngRequest {
            capture: CaptureInput {
                capture_path: "captures/frame.rdc".to_string(),
            },
            event_id: Some(42),
            texture_index: 3,
            output: OutputFile {
                output_path: "artifacts/frame.png".to_string(),
            },
        };

        let resolved = req.resolved_in_cwd(Path::new("/tmp/project"));

        assert_eq!(
            resolved.capture.capture_path,
            "/tmp/project/captures/frame.rdc"
        );
        assert_eq!(resolved.event_id, Some(42));
        assert_eq!(resolved.texture_index, 3);
        assert_eq!(
            resolved.output.output_path,
            "/tmp/project/artifacts/frame.png"
        );
    }

    #[test]
    fn replay_save_texture_request_serializes_capture_and_output_flattened() {
        let req = ReplaySaveTexturePngRequest {
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            event_id: Some(42),
            texture_index: 3,
            output: OutputFile {
                output_path: "/tmp/frame.png".to_string(),
            },
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("event_id"), Some(&Value::Number(42_u32.into())));
        assert_eq!(
            object.get("texture_index"),
            Some(&Value::Number(3_u32.into()))
        );
        assert_eq!(
            object.get("output_path"),
            Some(&Value::String("/tmp/frame.png".to_string()))
        );
        assert!(!object.contains_key("capture"));
        assert!(!object.contains_key("output"));
    }

    #[test]
    fn replay_save_outputs_request_normalizes_via_shared_export_output() {
        let req = ReplaySaveOutputsPngRequest {
            capture: CaptureInput {
                capture_path: "captures/frame.rdc".to_string(),
            },
            event_id: Some(42),
            output: ExportOutput::default(),
            include_depth: true,
        };

        let normalized = req
            .normalized_in_cwd(Path::new("/tmp/project"))
            .expect("normalize should succeed");

        assert_eq!(
            normalized.capture.capture_path,
            "/tmp/project/captures/frame.rdc"
        );
        assert_eq!(
            normalized.output.output_dir.as_deref(),
            Some("/tmp/project/artifacts/renderdoc/exports")
        );
        assert_eq!(normalized.output.basename.as_deref(), Some("frame"));
        assert_eq!(normalized.event_id, Some(42));
        assert!(normalized.include_depth);
    }

    #[test]
    fn replay_save_outputs_request_serializes_capture_and_output_flattened() {
        let req = ReplaySaveOutputsPngRequest {
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            event_id: Some(42),
            output: ExportOutput {
                output_dir: Some("/tmp/out".to_string()),
                basename: Some("frame".to_string()),
            },
            include_depth: true,
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("event_id"), Some(&Value::Number(42_u32.into())));
        assert_eq!(
            object.get("output_dir"),
            Some(&Value::String("/tmp/out".to_string()))
        );
        assert_eq!(
            object.get("basename"),
            Some(&Value::String("frame".to_string()))
        );
        assert_eq!(object.get("include_depth"), Some(&Value::Bool(true)));
        assert!(!object.contains_key("capture"));
        assert!(!object.contains_key("output"));
    }
}
