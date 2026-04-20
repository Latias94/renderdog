use renderdog_automation as renderdog;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplayListTexturesRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ReplayListTexturesRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplayPickPixelRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ReplayPickPixelRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplaySaveTexturePngRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ReplaySaveTexturePngRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplaySaveOutputsPngRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    #[serde(flatten)]
    pub(crate) inner: renderdog::ReplaySaveOutputsPngRequest,
}
