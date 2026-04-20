use renderdog_automation as renderdog;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CaptureAndExportActionsRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::CaptureAndExportActionsRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CaptureAndExportBindingsIndexRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::CaptureAndExportBindingsIndexRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CaptureAndExportBundleRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::CaptureAndExportBundleRequest,
}

pub(crate) type CaptureAndExportActionsResponse = renderdog::CaptureAndExportActionsResponse;
pub(crate) type CaptureAndExportBindingsIndexResponse =
    renderdog::CaptureAndExportBindingsIndexResponse;
pub(crate) type CaptureAndExportBundleResponse = renderdog::CaptureAndExportBundleResponse;
