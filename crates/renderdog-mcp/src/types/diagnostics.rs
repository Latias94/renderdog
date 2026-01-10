use schemars::JsonSchema;
use serde::Serialize;

use renderdog_automation as renderdog;

#[derive(Debug, Serialize, JsonSchema)]
pub(crate) struct DetectInstallationResponse {
    pub(crate) root_dir: String,
    pub(crate) qrenderdoc_exe: String,
    pub(crate) renderdoccmd_exe: String,
    pub(crate) version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) vulkan_layer: Option<renderdog::VulkanLayerDiagnosis>,
}
