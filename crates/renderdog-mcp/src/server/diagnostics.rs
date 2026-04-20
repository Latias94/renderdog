use std::time::Instant;

use rmcp::{Json, tool, tool_router};

use renderdog_automation as renderdog;

use crate::types::DetectInstallationResponse;

use super::{RenderdogMcpServer, require_installation, tool_result};

#[tool_router(router = diagnostics_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_detect_installation",
        description = "Detect local RenderDoc installation and return tool paths."
    )]
    async fn detect_installation(&self) -> Result<Json<DetectInstallationResponse>, String> {
        let tool = "renderdoc_detect_installation";
        let start = Instant::now();
        tracing::info!(tool = tool, "start");
        let install = require_installation(tool)?;

        let version = install.version().ok().map(|s| s.trim().to_string());
        let vulkan_layer = install.diagnose_vulkan_layer().ok();

        tracing::info!(tool = tool, elapsed_ms = start.elapsed().as_millis(), "ok");
        Ok(Json(DetectInstallationResponse {
            root_dir: install.root_dir.display().to_string(),
            qrenderdoc_exe: install.qrenderdoc_exe.display().to_string(),
            renderdoccmd_exe: install.renderdoccmd_exe.display().to_string(),
            version,
            vulkan_layer,
        }))
    }

    #[tool(
        name = "renderdoc_vulkanlayer_diagnose",
        description = "Diagnose Vulkan layer registration status using `renderdoccmd vulkanlayer --explain` and return suggested fix commands."
    )]
    async fn vulkanlayer_diagnose(&self) -> Result<Json<renderdog::VulkanLayerDiagnosis>, String> {
        let tool = "renderdoc_vulkanlayer_diagnose";
        let start = Instant::now();
        tracing::info!(tool = tool, "start");
        let install = require_installation(tool)?;
        let diag = tool_result(
            tool,
            "diagnose vulkan layer",
            install.diagnose_vulkan_layer(),
        )?;
        tracing::info!(tool = tool, elapsed_ms = start.elapsed().as_millis(), "ok");
        Ok(Json(diag))
    }

    #[tool(
        name = "renderdoc_diagnose_environment",
        description = "Diagnose RenderDoc environment (paths, renderdoccmd version, Vulkan layer registration, and key Vulkan-related env vars) and return warnings + suggested fixes."
    )]
    async fn diagnose_environment(&self) -> Result<Json<renderdog::EnvironmentDiagnosis>, String> {
        let tool = "renderdoc_diagnose_environment";
        let start = Instant::now();
        tracing::info!(tool = tool, "start");
        let install = require_installation(tool)?;
        let diag = tool_result(tool, "diagnose environment", install.diagnose_environment())?;
        tracing::info!(tool = tool, elapsed_ms = start.elapsed().as_millis(), "ok");
        Ok(Json(diag))
    }
}
