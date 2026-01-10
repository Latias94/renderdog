use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplayListTexturesRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) event_id: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplayPickPixelRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) event_id: Option<u32>,
    pub(crate) texture_index: u32,
    pub(crate) x: u32,
    pub(crate) y: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplaySaveTexturePngRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) event_id: Option<u32>,
    pub(crate) texture_index: u32,
    pub(crate) output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ReplaySaveOutputsPngRequest {
    #[serde(default)]
    pub(crate) cwd: Option<String>,
    pub(crate) capture_path: String,
    #[serde(default)]
    pub(crate) event_id: Option<u32>,
    #[serde(default)]
    pub(crate) output_dir: Option<String>,
    #[serde(default)]
    pub(crate) basename: Option<String>,
    #[serde(default)]
    pub(crate) include_depth: bool,
}
