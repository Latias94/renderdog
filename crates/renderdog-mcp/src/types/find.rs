use schemars::JsonSchema;
use serde::Deserialize;

use renderdog_automation as renderdog;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct FindEventsRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::FindEventsRequest,
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
