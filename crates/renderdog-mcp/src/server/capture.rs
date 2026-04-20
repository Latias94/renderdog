use std::{ffi::OsString, time::Instant};

use rmcp::{Json, handler::server::wrapper::Parameters, tool, tool_router};

use renderdog_automation as renderdog;

use crate::{
    paths::resolve_path_from_base,
    types::{
        LaunchCaptureRequest, LaunchCaptureResponse, OpenCaptureUiRequest, OpenCaptureUiResponse,
        SaveThumbnailRequest, SaveThumbnailResponse,
        TriggerCaptureRequest as TriggerCaptureToolRequest,
    },
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

        let cwd = req.resolve_cwd()?;
        let req = req.inner;

        let artifacts_dir = req
            .artifacts_dir
            .as_deref()
            .map(|path| resolve_path_from_base(&cwd, path))
            .unwrap_or_else(|| renderdog::default_artifacts_dir(&cwd));

        std::fs::create_dir_all(&artifacts_dir)
            .map_err(|err| format!("create artifacts_dir failed: {err}"))?;

        let capture_file_template = req
            .capture_template_name
            .as_deref()
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        let request = renderdog::CaptureLaunchRequest {
            executable: resolve_path_from_base(&cwd, &req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req
                .working_dir
                .map(|path| resolve_path_from_base(&cwd, &path)),
            capture_file_template: capture_file_template.clone(),
        };

        let res = tool_result(tool, "launch capture", install.launch_capture(&request))?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = res.target_ident,
            "ok"
        );
        Ok(Json(LaunchCaptureResponse {
            target_ident: res.target_ident,
            capture_file_template: capture_file_template.map(|path| path.display().to_string()),
            stdout: res.stdout,
            stderr: res.stderr,
        }))
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
        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);
        let output_path = resolve_path_from_base(&cwd, &req.output_path);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("create output dir failed: {err}"))?;
        }

        tool_result(
            tool,
            "save thumbnail",
            install.save_thumbnail(&capture_path, &output_path),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            output_path = %output_path.display(),
            "ok"
        );
        Ok(Json(SaveThumbnailResponse {
            output_path: output_path.display().to_string(),
        }))
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
        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);

        let child = tool_result(
            tool,
            "open capture UI",
            install.open_capture_in_ui(&capture_path),
        )?;

        let pid = child.id();
        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            pid,
            "ok"
        );
        Ok(Json(OpenCaptureUiResponse {
            capture_path: capture_path.display().to_string(),
            pid,
        }))
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
            install.trigger_capture_via_target_control(
                &cwd,
                &renderdog::TriggerCaptureRequest {
                    host: req.host,
                    target_ident: req.target_ident,
                    num_frames: req.num_frames,
                    timeout_s: req.timeout_s,
                },
            ),
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
