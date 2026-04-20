use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use renderdog_automation as renderdog;

use super::CwdRequest;

pub(crate) type ExportActionsRequest = CwdRequest<renderdog::ExportActionsRequest>;
pub(crate) type ExportBindingsIndexRequest = CwdRequest<renderdog::ExportBindingsIndexRequest>;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ExportBundleInput {
    #[serde(default)]
    pub(crate) save_thumbnail: bool,
    #[serde(default)]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub(crate) open_capture_ui: bool,
    #[serde(flatten)]
    pub(crate) export: renderdog::ExportBundleRequest,
}

pub(crate) type ExportBundleRequest = CwdRequest<ExportBundleInput>;

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct ExportBundleResponse {
    pub(crate) bundle: renderdog::ExportBundleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ui_pid: Option<u32>,
}
