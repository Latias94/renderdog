//! High-level RenderDoc workflows built on `qrenderdoc --python`.

mod export_actions;
mod export_bindings_index;
mod export_bundle;
mod find_and_save_outputs;
mod find_events;
mod one_shot;
mod trigger_capture;

pub use export_actions::ExportActionsError;
pub use export_bindings_index::ExportBindingsIndexError;
pub use export_bundle::ExportBundleError;
pub use find_and_save_outputs::{
    FindEventSelection, FindEventsAndSaveOutputsPngError, FindEventsAndSaveOutputsPngRequest,
    FindEventsAndSaveOutputsPngResponse,
};
pub use find_events::FindEventsError;
pub use one_shot::{
    CaptureAndExportActionsError, CaptureAndExportActionsRequest, CaptureAndExportActionsResponse,
    CaptureAndExportBindingsIndexError, CaptureAndExportBindingsIndexRequest,
    CaptureAndExportBindingsIndexResponse, CaptureAndExportBundleError,
    CaptureAndExportBundleRequest, CaptureAndExportBundleResponse, PrepareOneShotCaptureError,
};
pub use trigger_capture::TriggerCaptureError;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportOutput {
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
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
pub struct OneShotCaptureOptions {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
}

impl Default for OneShotCaptureOptions {
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
    pub host: String,
    pub target_ident: u32,
    pub num_frames: u32,
    pub timeout_s: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TriggerCaptureResponse {
    pub capture_path: String,
    pub frame_number: u32,
    pub api: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportActionsRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportActionsResponse {
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
pub struct ExportBindingsIndexRequest {
    #[serde(flatten)]
    pub capture: CaptureInput,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportBindingsIndexResponse {
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
}
