use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use renderdog_automation as renderdog;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct FindEventsRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
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
    #[serde(default = "crate::types::defaults::default_max_results")]
    pub(crate) max_results: Option<u32>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FindEventSelection {
    First,
    #[default]
    Last,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct FindEventsAndSaveOutputsPngRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,

    #[serde(default)]
    pub(crate) selection: FindEventSelection,

    #[serde(default = "crate::types::defaults::default_true")]
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
    #[serde(default = "crate::types::defaults::default_max_results")]
    pub(crate) max_results: Option<u32>,

    #[serde(default)]
    pub(crate) output_dir: Option<String>,
    #[serde(default)]
    pub(crate) basename: Option<String>,
    #[serde(default)]
    pub(crate) include_depth: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct FindEventsAndSaveOutputsPngResponse {
    pub(crate) find: renderdog::FindEventsResponse,
    pub(crate) selected_event_id: u32,
    pub(crate) replay: renderdog::ReplaySaveOutputsPngResponse,
}
