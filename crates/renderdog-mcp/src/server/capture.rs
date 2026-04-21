use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use super::{CwdRequest, RenderdogMcpServer, ToolRun};

#[tool_router(router = capture_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_save_thumbnail",
        description = "Extract embedded thumbnail from a .rdc capture using renderdoccmd thumb."
    )]
    async fn save_thumbnail(
        &self,
        Parameters(req): Parameters<CwdRequest<renderdog::SaveThumbnailRequest>>,
    ) -> Result<Json<renderdog::SaveThumbnailResponse>, String> {
        let tool = "renderdoc_save_thumbnail";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("save thumbnail", req, |install, cwd, req| {
            install.save_thumbnail_in_cwd(&cwd, &req)
        })?;

        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            output_path = %res.output.output_path,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_open_capture_ui",
        description = "Open a .rdc capture in qrenderdoc UI."
    )]
    async fn open_capture_ui(
        &self,
        Parameters(req): Parameters<CwdRequest<renderdog::OpenCaptureUiRequest>>,
    ) -> Result<Json<renderdog::OpenCaptureUiResponse>, String> {
        let tool = "renderdoc_open_capture_ui";
        let run = ToolRun::start(tool, || {
            tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        });
        let res = run.with_install_and_cwd("open capture UI", req, |install, cwd, req| {
            install.open_capture_ui_in_cwd(&cwd, &req)
        })?;
        tracing::info!(
            tool = tool,
            elapsed_ms = run.elapsed_ms(),
            pid = res.pid,
            "ok"
        );
        Ok(Json(res))
    }
}
