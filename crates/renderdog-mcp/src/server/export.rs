use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use super::{CwdRequest, RenderdogMcpServer, ToolRun};

#[tool_router(router = export_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_export_bundle_jsonl",
        description = "Export both actions + bindings index from an existing .rdc capture, and optionally save a thumbnail/open qrenderdoc UI."
    )]
    async fn export_bundle_tool(
        &self,
        Parameters(req): Parameters<CwdRequest<renderdog::ExportBundleRequest>>,
    ) -> Result<Json<renderdog::ExportBundleResponse>, String> {
        let tool = "renderdoc_export_bundle_jsonl";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("export bundle", req, |install, cwd, req| {
            install.export_bundle(&cwd, &req)
        })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            capture_path = %res.capture.capture_path,
            actions_jsonl_path = %res.artifacts.actions.actions_jsonl_path,
            bindings_jsonl_path = %res.artifacts.bindings.bindings_jsonl_path,
            total_actions = res.artifacts.actions.total_actions,
            total_drawcalls = res.artifacts.bindings.total_drawcalls,
            "ok"
        );

        Ok(Json(res))
    }
}
