use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct LaunchCaptureRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) executable: String,
    #[serde(default)]
    pub(crate) args: Vec<String>,
    #[serde(default)]
    pub(crate) working_dir: Option<String>,
    #[serde(default)]
    pub(crate) artifacts_dir: Option<String>,
    #[serde(default)]
    pub(crate) capture_template_name: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct LaunchCaptureResponse {
    pub(crate) target_ident: u32,
    pub(crate) capture_file_template: Option<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct SaveThumbnailRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    pub(crate) output_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct SaveThumbnailResponse {
    pub(crate) output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct OpenCaptureUiRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct OpenCaptureUiResponse {
    pub(crate) capture_path: String,
    pub(crate) pid: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct TriggerCaptureRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(default = "crate::types::defaults::default_host")]
    pub(crate) host: String,
    pub(crate) target_ident: u32,
    #[serde(default = "crate::types::defaults::default_frames")]
    pub(crate) num_frames: u32,
    #[serde(default = "crate::types::defaults::default_timeout_s")]
    pub(crate) timeout_s: u32,
}
