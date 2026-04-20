//! High-level RenderDoc workflows built on `qrenderdoc --python`.

mod export_actions;
mod export_bindings_index;
mod export_bundle;
mod find_and_save_outputs;
mod find_events;
mod one_shot;
mod trigger_capture;

pub use export_bundle::ExportBundleError;
pub use find_and_save_outputs::{
    FindEventSelection, FindEventsAndSaveOutputsPngError, FindEventsAndSaveOutputsPngRequest,
    FindEventsAndSaveOutputsPngResponse,
};
pub use find_events::FindEventsError;
pub use one_shot::{
    CaptureAndExportBundleError, CaptureAndExportBundleRequest, CaptureAndExportBundleResponse,
    PrepareOneShotCaptureError,
};
pub use trigger_capture::TriggerCaptureError;

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::scripting::PrepareQRenderDocJsonRequest;
use crate::{QRenderDocJsonError, normalize_capture_path, prepare_export_target};

fn default_max_results() -> Option<u32> {
    Some(200)
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_frames() -> u32 {
    1
}

fn default_timeout_s() -> u32 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureInput {
    pub capture_path: String,
}

impl CaptureInput {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture_path: normalize_capture_path(cwd, &self.capture_path),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportOutput {
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
}

impl ExportOutput {
    pub(crate) fn normalized_for_capture(
        &self,
        cwd: &Path,
        capture: &CaptureInput,
    ) -> Result<(CaptureInput, Self), std::io::Error> {
        let prepared = prepare_export_target(
            cwd,
            &capture.capture_path,
            self.output_dir.as_deref(),
            self.basename.as_deref(),
        )?;

        Ok((
            CaptureInput {
                capture_path: prepared.capture_path,
            },
            Self {
                output_dir: Some(prepared.output_dir),
                basename: Some(prepared.basename),
            },
        ))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapturePostActions {
    #[serde(default)]
    pub save_thumbnail: bool,
    #[serde(default)]
    pub thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub open_capture_ui: bool,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CapturePostActionOutputs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_pid: Option<u32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventFilter {
    #[serde(default)]
    pub marker_prefix: Option<String>,
    #[serde(default)]
    pub event_id_min: Option<u32>,
    #[serde(default)]
    pub event_id_max: Option<u32>,
    #[serde(default)]
    pub name_contains: Option<String>,
    #[serde(default)]
    pub marker_contains: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct DrawcallScope {
    #[serde(default)]
    pub only_drawcalls: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsLimit {
    #[serde(default = "default_max_results")]
    pub max_results: Option<u32>,
}

impl Default for FindEventsLimit {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct BindingsExportOptions {
    #[serde(default)]
    pub include_cbuffers: bool,
    #[serde(default)]
    pub include_outputs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneShotCaptureTarget {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub artifacts_dir: Option<String>,
    #[serde(default)]
    pub capture_template_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneShotTriggerOptions {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
}

impl Default for OneShotTriggerOptions {
    fn default() -> Self {
        Self {
            host: default_host(),
            num_frames: default_frames(),
            timeout_s: default_timeout_s(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureRequest {
    #[serde(default = "default_host")]
    pub host: String,
    pub target_ident: u32,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
}

impl PrepareQRenderDocJsonRequest for TriggerCaptureRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, _cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureResponse {
    pub capture_path: String,
    pub frame_number: u32,
    pub api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportActionsRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
}

impl PrepareQRenderDocJsonRequest for ExportActionsRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, _cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportActionsResponse {
    pub capture_path: String,
    pub actions_jsonl_path: String,
    pub summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub limit: FindEventsLimit,
}

impl FindEventsRequest {
    pub(crate) fn normalized_in_cwd(&self, cwd: &Path) -> Self {
        Self {
            capture: self.capture.normalized_in_cwd(cwd),
            ..self.clone()
        }
    }
}

impl PrepareQRenderDocJsonRequest for FindEventsRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.normalized_in_cwd(cwd))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FoundEvent {
    pub event_id: u32,
    pub parent_event_id: Option<u32>,
    pub depth: u32,
    pub name: String,
    pub flags: u64,
    pub flags_names: Vec<String>,
    pub marker_path: Vec<String>,
    pub marker_path_joined: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindEventsResponse {
    pub capture_path: String,
    pub total_matches: u64,
    pub truncated: bool,
    pub first_event_id: Option<u32>,
    pub last_event_id: Option<u32>,
    pub matches: Vec<FoundEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportBindingsIndexRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
}

impl PrepareQRenderDocJsonRequest for ExportBindingsIndexRequest {
    type Error = QRenderDocJsonError;

    fn prepare_in_cwd(&self, _cwd: &Path) -> Result<Self, Self::Error> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub(crate) struct ExportBindingsIndexResponse {
    pub capture_path: String,
    pub bindings_jsonl_path: String,
    pub summary_json_path: String,
    pub total_drawcalls: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBundleRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
    #[serde(flatten)]
    pub post_actions: CapturePostActions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBundleResponse {
    pub capture_path: String,

    pub actions_jsonl_path: String,
    pub actions_summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,

    pub bindings_jsonl_path: String,
    pub bindings_summary_json_path: String,
    pub total_drawcalls: u64,
    #[serde(flatten)]
    pub post_actions: CapturePostActionOutputs,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{CaptureInput, ExportOutput};

    #[test]
    fn capture_input_normalizes_relative_path_in_cwd() {
        let capture = CaptureInput {
            capture_path: "captures/frame.rdc".to_string(),
        };

        let normalized = capture.normalized_in_cwd(Path::new("/tmp/project"));

        assert_eq!(normalized.capture_path, "/tmp/project/captures/frame.rdc");
    }

    #[test]
    fn export_output_normalization_uses_capture_basename_when_missing() {
        let capture = CaptureInput {
            capture_path: "captures/frame.rdc".to_string(),
        };
        let output = ExportOutput::default();

        let (capture, output) = output
            .normalized_for_capture(Path::new("/tmp/project"), &capture)
            .expect("normalize export target");

        assert_eq!(capture.capture_path, "/tmp/project/captures/frame.rdc");
        assert_eq!(
            output.output_dir.as_deref(),
            Some("/tmp/project/artifacts/renderdoc/exports")
        );
        assert_eq!(output.basename.as_deref(), Some("frame"));
    }
}
