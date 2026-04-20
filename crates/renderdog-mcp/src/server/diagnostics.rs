use rmcp::{Json, tool, tool_router};

use renderdog_automation as renderdog;

use super::{RenderdogMcpServer, ToolRun};

#[tool_router(router = diagnostics_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_detect_installation",
        description = "Detect the local RenderDoc installation, resolve tool paths, and probe renderdoccmd version, workspace replay header compatibility, and Vulkan layer status. Probe failures are returned in `*_error` fields instead of failing the tool."
    )]
    async fn detect_installation(&self) -> Result<Json<renderdog::InstallationDetection>, String> {
        let tool = "renderdoc_detect_installation";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, "start");
        });
        let res = run.with_install("detect installation", |install| {
            Ok::<_, std::convert::Infallible>(install.describe_installation())
        })?;

        tracing::info!(tool = tool, elapsed_ms = run.elapsed_ms(), "ok");
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_vulkanlayer_diagnose",
        description = "Run `renderdoccmd vulkanlayer --explain`, parse Vulkan layer registration status, and return raw command output plus suggested fix commands."
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
        description = "Diagnose the RenderDoc environment by combining installation detection with platform, arch, elevation, discovered Vulkan layer manifests, key Vulkan-related env vars, aggregated warnings, and suggested fix commands."
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
