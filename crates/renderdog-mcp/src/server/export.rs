use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use crate::types::{
    ExportActionsRequest as ExportActionsToolRequest,
    ExportBindingsIndexRequest as ExportBindingsIndexToolRequest,
    ExportBundleRequest as ExportBundleToolRequest, ExportBundleResponse,
};

use super::{RenderdogMcpServer, ToolRun};

#[tool_router(router = export_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_export_actions_jsonl",
        description = "Export an action/event tree from a .rdc capture into a searchable JSONL format via `qrenderdoc --python`."
    )]
    async fn export_actions_jsonl(
        &self,
        Parameters(req): Parameters<ExportActionsToolRequest>,
    ) -> Result<Json<renderdog_automation::ExportActionsResponse>, String> {
        let tool = "renderdoc_export_actions_jsonl";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("export actions", req, |install, cwd, req| {
            install.export_actions_jsonl(&cwd, &req)
        })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            actions_jsonl_path = %res.actions_jsonl_path,
            total_actions = res.total_actions,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_export_bindings_index_jsonl",
        description = "Export a searchable bindings index (`*.bindings.jsonl`) via `qrenderdoc --python` for fast offline querying."
    )]
    async fn export_bindings_index_jsonl(
        &self,
        Parameters(req): Parameters<ExportBindingsIndexToolRequest>,
    ) -> Result<Json<renderdog_automation::ExportBindingsIndexResponse>, String> {
        let tool = "renderdoc_export_bindings_index_jsonl";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("export bindings index", req, |install, cwd, req| {
            install.export_bindings_index_jsonl(&cwd, &req)
        })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            bindings_jsonl_path = %res.bindings_jsonl_path,
            total_drawcalls = res.total_drawcalls,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_export_bundle_jsonl",
        description = "Export both actions + bindings index from an existing .rdc capture, and optionally save a thumbnail/open qrenderdoc UI."
    )]
    async fn export_bundle_jsonl(
        &self,
        Parameters(req): Parameters<ExportBundleToolRequest>,
    ) -> Result<Json<ExportBundleResponse>, String> {
        let tool = "renderdoc_export_bundle_jsonl";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("export bundle", req, |install, cwd, req| {
            install.export_bundle_jsonl(&cwd, &req)
        })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
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
