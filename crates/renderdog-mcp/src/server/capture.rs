use std::time::Instant;

use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use crate::types::{
    LaunchCaptureRequest, LaunchCaptureResponse, OpenCaptureUiRequest, OpenCaptureUiResponse,
    SaveThumbnailRequest, SaveThumbnailResponse,
    TriggerCaptureRequest as TriggerCaptureToolRequest,
};

use super::{RenderdogMcpServer, require_installation, tool_result};

#[tool_router(router = capture_tool_router, vis = "pub(super)")]
impl RenderdogMcpServer {
    #[tool(
        name = "renderdoc_launch_capture",
        description = "Launch target executable under RenderDoc injection using renderdoccmd capture; returns target ident (port)."
    )]
    async fn launch_capture(
        &self,
        Parameters(req): Parameters<LaunchCaptureRequest>,
    ) -> Result<Json<LaunchCaptureResponse>, String> {
        let tool = "renderdoc_launch_capture";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            executable = %req.inner.executable,
            args_len = req.inner.args.len(),
            "start"
        );
        let install = require_installation(tool)?;

        let (cwd, req) = req.into_parts()?;
        let res = tool_result(
            tool,
            "launch capture",
            install.launch_capture_in_cwd(&cwd, &req),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = res.target_ident,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_save_thumbnail",
        description = "Extract embedded thumbnail from a .rdc capture using renderdoccmd thumb."
    )]
    async fn save_thumbnail(
        &self,
        Parameters(req): Parameters<SaveThumbnailRequest>,
    ) -> Result<Json<SaveThumbnailResponse>, String> {
        let tool = "renderdoc_save_thumbnail";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture_path, "start");
        let install = require_installation(tool)?;

        let (cwd, req) = req.into_parts()?;
        let res = tool_result(
            tool,
            "save thumbnail",
            install.save_thumbnail_in_cwd(&cwd, &req),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            output_path = %res.output_path,
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
        Parameters(req): Parameters<OpenCaptureUiRequest>,
    ) -> Result<Json<OpenCaptureUiResponse>, String> {
        let tool = "renderdoc_open_capture_ui";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture_path, "start");
        let install = require_installation(tool)?;

        let (cwd, req) = req.into_parts()?;
        let res = tool_result(
            tool,
            "open capture UI",
            install.open_capture_ui_in_cwd(&cwd, &req),
        )?;
        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            pid = res.pid,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_trigger_capture",
        description = "Trigger a frame capture on a RenderDoc-injected target (started via renderdoccmd capture) and return the resulting .rdc path."
    )]
    async fn trigger_capture(
        &self,
        Parameters(req): Parameters<TriggerCaptureToolRequest>,
    ) -> Result<Json<renderdog::TriggerCaptureResponse>, String> {
        let tool = "renderdoc_trigger_capture";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            target_ident = req.inner.target_ident,
            num_frames = req.inner.num_frames,
            "start"
        );
        let install = require_installation(tool)?;

        let (cwd, req) = req.into_parts()?;
        let res = tool_result(
            tool,
            "trigger capture",
            install.trigger_capture_via_target_control(&cwd, &req),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            capture_path = %res.capture_path,
            "ok"
        );
        Ok(Json(res))
    }
}
