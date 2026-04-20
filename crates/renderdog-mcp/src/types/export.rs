use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use renderdog_automation as renderdog;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportActionsRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ExportActionsRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportBindingsIndexRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ExportBindingsIndexRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportBundleRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(default)]
    pub(crate) save_thumbnail: bool,
    #[serde(default)]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub(crate) open_capture_ui: bool,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ExportBundleRequest,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct ExportBundleResponse {
    pub(crate) bundle: renderdog::ExportBundleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ui_pid: Option<u32>,
}
