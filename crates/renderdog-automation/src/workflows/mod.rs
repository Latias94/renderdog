//! High-level RenderDoc workflows built on `qrenderdoc --python`.

mod export_actions;
mod export_bindings_index;
mod export_bundle;
mod find_events;
mod trigger_capture;
mod util;

pub use export_actions::ExportActionsError;
pub use export_bindings_index::ExportBindingsIndexError;
pub use export_bundle::ExportBundleError;
pub use find_events::FindEventsError;
pub use trigger_capture::TriggerCaptureError;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,
    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
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
    pub capture_path: String,
    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
    pub max_results: Option<u32>,
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
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,
    pub include_cbuffers: bool,
    pub include_outputs: bool,
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
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,

    pub only_drawcalls: bool,
    pub marker_prefix: Option<String>,
    pub event_id_min: Option<u32>,
    pub event_id_max: Option<u32>,
    pub name_contains: Option<String>,
    pub marker_contains: Option<String>,
    pub case_sensitive: bool,

    pub include_cbuffers: bool,
    pub include_outputs: bool,
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
