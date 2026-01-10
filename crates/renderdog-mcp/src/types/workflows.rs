use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CaptureAndExportActionsRequest {
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

    #[serde(default = "crate::types::defaults::default_host")]
    pub(crate) host: String,
    #[serde(default = "crate::types::defaults::default_frames")]
    pub(crate) num_frames: u32,
    #[serde(default = "crate::types::defaults::default_timeout_s")]
    pub(crate) timeout_s: u32,

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
pub(crate) struct CaptureAndExportBindingsIndexRequest {
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

    #[serde(default = "crate::types::defaults::default_host")]
    pub(crate) host: String,
    #[serde(default = "crate::types::defaults::default_frames")]
    pub(crate) num_frames: u32,
    #[serde(default = "crate::types::defaults::default_timeout_s")]
    pub(crate) timeout_s: u32,

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

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct CaptureAndExportBindingsIndexResponse {
    pub(crate) target_ident: u32,
    pub(crate) capture_path: String,
    pub(crate) capture_file_template: Option<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,

    pub(crate) bindings_jsonl_path: String,
    pub(crate) summary_json_path: String,
    pub(crate) total_drawcalls: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CaptureAndExportBundleRequest {
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

    #[serde(default = "crate::types::defaults::default_host")]
    pub(crate) host: String,
    #[serde(default = "crate::types::defaults::default_frames")]
    pub(crate) num_frames: u32,
    #[serde(default = "crate::types::defaults::default_timeout_s")]
    pub(crate) timeout_s: u32,

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

    #[serde(default)]
    pub(crate) include_cbuffers: bool,
    #[serde(default)]
    pub(crate) include_outputs: bool,

    #[serde(default)]
    pub(crate) save_thumbnail: bool,
    #[serde(default)]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub(crate) open_capture_ui: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct CaptureAndExportBundleResponse {
    pub(crate) target_ident: u32,
    pub(crate) capture_path: String,
    pub(crate) capture_file_template: Option<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,

    pub(crate) actions_jsonl_path: String,
    pub(crate) actions_summary_json_path: String,
    pub(crate) total_actions: u64,
    pub(crate) drawcall_actions: u64,

    pub(crate) bindings_jsonl_path: String,
    pub(crate) bindings_summary_json_path: String,
    pub(crate) total_drawcalls: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ui_pid: Option<u32>,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct CaptureAndExportActionsResponse {
    pub(crate) target_ident: u32,
    pub(crate) capture_path: String,
    pub(crate) capture_file_template: Option<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,

    pub(crate) actions_jsonl_path: String,
    pub(crate) summary_json_path: String,
    pub(crate) total_actions: u64,
    pub(crate) drawcall_actions: u64,
}
