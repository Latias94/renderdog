use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use crate::types::{
    CaptureAndExportBundleRequest as CaptureAndExportBundleToolRequest,
    CaptureAndExportBundleResponse,
};

use super::{RenderdogMcpServer, ToolRun};

#[tool_router(router = workflows_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_capture_and_export_bundle_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export <basename>.actions.jsonl (+ summary) and <basename>.bindings.jsonl (+ bindings_summary)."
    )]
    async fn capture_and_export_bundle_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportBundleToolRequest>,
    ) -> Result<Json<CaptureAndExportBundleResponse>, String> {
        let tool = "renderdoc_capture_and_export_bundle_jsonl";
        let run = ToolRun::start(tool, || {
            tracing::info!(
                tool = tool,
                executable = %req.inner.target.executable,
                args_len = req.inner.target.args.len(),
                "start"
            );
        });
        let res = run.with_install_and_cwd(
            "one-shot capture/export bundle",
            req,
            |install, cwd, req| install.capture_and_export_bundle_jsonl(&cwd, &req),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            target_ident = res.target_ident,
            capture_path = %res.capture_path,
            actions_jsonl_path = %res.actions_jsonl_path,
            bindings_jsonl_path = %res.bindings_jsonl_path,
            total_actions = res.total_actions,
            total_drawcalls = res.total_drawcalls,
            "ok"
        );

        Ok(Json(res))
    }
}
