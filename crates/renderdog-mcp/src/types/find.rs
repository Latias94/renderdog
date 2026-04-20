use schemars::JsonSchema;
use serde::Deserialize;

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

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct FindEventsAndSaveOutputsPngRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::FindEventsAndSaveOutputsPngRequest,
}

pub(crate) type FindEventsAndSaveOutputsPngResponse =
    renderdog::FindEventsAndSaveOutputsPngResponse;
