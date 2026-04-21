use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::QRenderDocJsonError;
use crate::qrenderdoc_jobs::{
    REPLAY_LIST_TEXTURES_JOB, REPLAY_PICK_PIXEL_JOB, REPLAY_SAVE_OUTPUTS_PNG_JOB,
    REPLAY_SAVE_TEXTURE_PNG_JOB,
};
use crate::scripting::PrepareQRenderDocJsonRequest;
use crate::{CaptureInput, CaptureRef, ExportOutput, OutputFile, OutputRef, RenderDocInstallation};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    pub event_id: Option<u32>,
}

impl ReplayListTexturesRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture: self.capture.normalized_in_cwd(cwd),
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
pub struct ReplayContext {
    #[serde(flatten)]
    pub capture: CaptureRef,
    pub event_id: Option<u32>,
}

#[cfg(test)]
impl ReplayContext {
    pub(crate) fn new(capture_path: impl Into<String>, event_id: Option<u32>) -> Self {
        Self {
            capture: CaptureRef::new(capture_path),
            event_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SelectedReplayContext {
    #[serde(flatten)]
    pub capture: CaptureRef,
    pub event_id: u32,
}

#[cfg(test)]
impl SelectedReplayContext {
    pub(crate) fn new(capture_path: impl Into<String>, event_id: u32) -> Self {
        Self {
            capture: CaptureRef::new(capture_path),
            event_id,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct ReplayTextureRef {
    pub texture_index: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct PixelPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayListTexturesResponse {
    #[serde(flatten)]
    pub context: ReplayContext,
    pub textures: Vec<ReplayTextureInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplayPickPixelRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    pub event_id: Option<u32>,
    pub texture_index: u32,
    pub x: u32,
    pub y: u32,
}

impl ReplayPickPixelRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture: self.capture.normalized_in_cwd(cwd),
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
    #[serde(flatten)]
    pub context: ReplayContext,
    #[serde(flatten)]
    pub texture: ReplayTextureRef,
    #[serde(flatten)]
    pub pixel: PixelPosition,
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
    #[serde(flatten)]
    pub context: ReplayContext,
    #[serde(flatten)]
    pub texture: ReplayTextureRef,
    #[serde(flatten)]
    pub output: OutputRef,
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
    #[serde(flatten)]
    pub output: OutputRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaySaveOutputsPngResponse {
    #[serde(flatten)]
    pub context: SelectedReplayContext,
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

    use super::{
        PixelPosition, ReplayContext, ReplayListTexturesRequest, ReplayListTexturesResponse,
        ReplayPickPixelRequest, ReplayPickPixelResponse, ReplaySaveOutputsPngRequest,
        ReplaySaveOutputsPngResponse, ReplaySaveTexturePngRequest, ReplaySaveTexturePngResponse,
        ReplaySavedImage, ReplayTextureInfo, ReplayTextureRef, SelectedReplayContext,
    };
    use crate::{CaptureInput, ExportOutput, OutputFile, OutputRef};

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
    fn replay_list_textures_request_serializes_capture_flattened() {
        let req = ReplayListTexturesRequest {
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            event_id: Some(42),
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("event_id"), Some(&Value::Number(42_u32.into())));
        assert!(!object.contains_key("capture"));
    }

    #[test]
    fn replay_pick_pixel_request_serializes_capture_flattened() {
        let req = ReplayPickPixelRequest {
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            event_id: Some(42),
            texture_index: 3,
            x: 10,
            y: 20,
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
        assert_eq!(object.get("x"), Some(&Value::Number(10_u32.into())));
        assert_eq!(object.get("y"), Some(&Value::Number(20_u32.into())));
        assert!(!object.contains_key("capture"));
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

    #[test]
    fn replay_list_textures_response_serializes_context_flattened() {
        let response = ReplayListTexturesResponse {
            context: ReplayContext::new("/tmp/frame.rdc", Some(42)),
            textures: vec![ReplayTextureInfo {
                index: 3,
                resource_id: 7,
                name: "Color".to_string(),
                width: 1920,
                height: 1080,
                depth: 1,
                mips: 1,
                arraysize: 1,
                ms_samp: 1,
                byte_size: 1024,
            }],
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("event_id"), Some(&Value::Number(42_u32.into())));
        assert!(object.get("textures").is_some());
        assert!(!object.contains_key("context"));
    }

    #[test]
    fn replay_pick_pixel_response_serializes_context_and_location_flattened() {
        let response = ReplayPickPixelResponse {
            context: ReplayContext::new("/tmp/frame.rdc", Some(42)),
            texture: ReplayTextureRef { texture_index: 3 },
            pixel: PixelPosition { x: 10, y: 20 },
            rgba: [0.1, 0.2, 0.3, 1.0],
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("event_id"), Some(&Value::Number(42_u32.into())));
        assert_eq!(
            object.get("texture_index"),
            Some(&Value::Number(3_u32.into()))
        );
        assert_eq!(object.get("x"), Some(&Value::Number(10_u32.into())));
        assert_eq!(object.get("y"), Some(&Value::Number(20_u32.into())));
        assert!(object.get("rgba").is_some());
        assert!(!object.contains_key("context"));
        assert!(!object.contains_key("texture"));
        assert!(!object.contains_key("pixel"));
    }

    #[test]
    fn replay_save_texture_response_serializes_context_and_output_flattened() {
        let response = ReplaySaveTexturePngResponse {
            context: ReplayContext::new("/tmp/frame.rdc", Some(42)),
            texture: ReplayTextureRef { texture_index: 3 },
            output: OutputRef::new("/tmp/frame.png"),
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

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
        assert!(!object.contains_key("context"));
        assert!(!object.contains_key("texture"));
        assert!(!object.contains_key("output"));
    }

    #[test]
    fn replay_save_outputs_response_serializes_context_and_saved_outputs_flattened() {
        let response = ReplaySaveOutputsPngResponse {
            context: SelectedReplayContext::new("/tmp/frame.rdc", 42),
            outputs: vec![ReplaySavedImage {
                kind: "color".to_string(),
                index: Some(0),
                resource_id: 7,
                output: OutputRef::new("/tmp/color0.png"),
            }],
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");
        let outputs = object
            .get("outputs")
            .and_then(Value::as_array)
            .expect("outputs array");
        let first = outputs
            .first()
            .and_then(Value::as_object)
            .expect("first output object");

        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(object.get("event_id"), Some(&Value::Number(42_u32.into())));
        assert_eq!(
            first.get("output_path"),
            Some(&Value::String("/tmp/color0.png".to_string()))
        );
        assert!(!object.contains_key("context"));
        assert!(!first.contains_key("output"));
    }
}
