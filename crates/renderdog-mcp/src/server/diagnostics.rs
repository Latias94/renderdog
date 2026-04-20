use rmcp::{Json, tool, tool_router};

use renderdog_automation as renderdog;

use crate::types::DetectInstallationResponse;

use super::{RenderdogMcpServer, ToolRun};

#[tool_router(router = diagnostics_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_detect_installation",
        description = "Detect local RenderDoc installation and return tool paths."
    )]
    async fn detect_installation(&self) -> Result<Json<DetectInstallationResponse>, String> {
        let tool = "renderdoc_detect_installation";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, "start");
        });
        let res = run.with_install("detect installation", |install| {
            let probe = install.probe_installation();
            Ok::<_, std::convert::Infallible>(DetectInstallationResponse {
                root_dir: install.root_dir.display().to_string(),
                qrenderdoc_exe: install.qrenderdoc_exe.display().to_string(),
                renderdoccmd_exe: install.renderdoccmd_exe.display().to_string(),
                renderdoccmd_version: probe.renderdoccmd_version,
                renderdoccmd_version_error: probe.renderdoccmd_version_error,
                workspace_renderdoc_version: probe.workspace_renderdoc_version,
                replay_version_match: probe.replay_version_match,
                vulkan_layer: probe.vulkan_layer,
                vulkan_layer_error: probe.vulkan_layer_error,
            })
        })?;

        tracing::info!(tool = tool, elapsed_ms = run.elapsed_ms(), "ok");
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_vulkanlayer_diagnose",
        description = "Diagnose Vulkan layer registration status using `renderdoccmd vulkanlayer --explain` and return suggested fix commands."
    )]
    async fn vulkanlayer_diagnose(&self) -> Result<Json<renderdog::VulkanLayerDiagnosis>, String> {
        let tool = "renderdoc_vulkanlayer_diagnose";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, "start");
        });
        let diag = run.with_install("diagnose vulkan layer", |install| {
            install.diagnose_vulkan_layer()
        })?;
        tracing::info!(tool = tool, elapsed_ms = run.elapsed_ms(), "ok");
        Ok(Json(diag))
    }

    #[tool(
        name = "renderdoc_diagnose_environment",
        description = "Diagnose RenderDoc environment (paths, installed renderdoccmd version, workspace replay header version, Vulkan layer registration, and key Vulkan-related env vars) and return warnings + suggested fixes."
    )]
    async fn diagnose_environment(&self) -> Result<Json<renderdog::EnvironmentDiagnosis>, String> {
        let tool = "renderdoc_diagnose_environment";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, "start");
        });
        let diag = run.with_install("diagnose environment", |install| {
            Ok::<_, std::convert::Infallible>(install.diagnose_environment())
        })?;
        tracing::info!(tool = tool, elapsed_ms = run.elapsed_ms(), "ok");
        Ok(Json(diag))
    }
}
