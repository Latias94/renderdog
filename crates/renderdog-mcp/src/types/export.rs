use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use renderdog_automation as renderdog;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportActionsRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) output_dir: Option<String>,
    #[serde(default)]
    pub(crate) basename: Option<String>,
    #[serde(default)]
    pub(crate) only_drawcalls: bool,
    #[serde(default)]
    pub(crate) marker_prefix: Option<String>,
    #[serde(default)]
    pub(crate) event_id_min: Option<u32>,
    #[serde(default)]
    pub(crate) event_id_max: Option<u32>,
    #[serde(default)]
    pub(crate) name_contains: Option<String>,
    #[serde(default)]
    pub(crate) marker_contains: Option<String>,
    #[serde(default)]
    pub(crate) case_sensitive: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportBindingsIndexRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) output_dir: Option<String>,
    #[serde(default)]
    pub(crate) basename: Option<String>,
    #[serde(default)]
    pub(crate) marker_prefix: Option<String>,
    #[serde(default)]
    pub(crate) event_id_min: Option<u32>,
    #[serde(default)]
    pub(crate) event_id_max: Option<u32>,
    #[serde(default)]
    pub(crate) name_contains: Option<String>,
    #[serde(default)]
    pub(crate) marker_contains: Option<String>,
    #[serde(default)]
    pub(crate) case_sensitive: bool,
    #[serde(default)]
    pub(crate) include_cbuffers: bool,
    #[serde(default)]
    pub(crate) include_outputs: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportBundleRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) output_dir: Option<String>,
    #[serde(default)]
    pub(crate) basename: Option<String>,

    #[serde(default)]
    pub(crate) save_thumbnail: bool,
    #[serde(default)]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub(crate) open_capture_ui: bool,

    #[serde(default)]
    pub(crate) only_drawcalls: bool,
    #[serde(default)]
    pub(crate) marker_prefix: Option<String>,
    #[serde(default)]
    pub(crate) event_id_min: Option<u32>,
    #[serde(default)]
    pub(crate) event_id_max: Option<u32>,
    #[serde(default)]
    pub(crate) name_contains: Option<String>,
    #[serde(default)]
    pub(crate) marker_contains: Option<String>,
    #[serde(default)]
    pub(crate) case_sensitive: bool,

    #[serde(default)]
    pub(crate) include_cbuffers: bool,
    #[serde(default)]
    pub(crate) include_outputs: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct ExportBundleResponse {
    pub(crate) bundle: renderdog::ExportBundleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ui_pid: Option<u32>,
}
