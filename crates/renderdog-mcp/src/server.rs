use std::{ffi::OsString, fmt::Display, path::Path, time::Instant};

use rmcp::{
    Json,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
};

use renderdog_automation as renderdog;

use crate::{
    paths::{resolve_base_cwd, resolve_path_from_base},
    types::*,
};

#[derive(Clone)]
pub(crate) struct RenderdogMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for RenderdogMcpServer {}

fn tool_result<T, E>(
    tool: &'static str,
    action: &'static str,
    result: Result<T, E>,
) -> Result<T, String>
where
    E: Display,
{
    result.map_err(|err| {
        tracing::error!(tool = tool, action = action, "failed");
        tracing::debug!(tool = tool, action = action, err = %err, "details");
        format!("{action} failed: {err}")
    })
}

fn require_installation(tool: &'static str) -> Result<renderdog::RenderDocInstallation, String> {
    tool_result(
        tool,
        "detect installation",
        renderdog::RenderDocInstallation::detect(),
    )
}

fn default_thumbnail_output_path(actions_jsonl_path: &str) -> String {
    let actions_path = Path::new(actions_jsonl_path);
    let basename = actions_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".actions.jsonl"))
        .or_else(|| actions_path.file_stem().and_then(|name| name.to_str()))
        .unwrap_or("capture");

    actions_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{basename}.thumb.png"))
        .display()
        .to_string()
}

#[tool_router(router = tool_router)]
impl RenderdogMcpServer {
    pub(crate) fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

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
            executable = %req.executable,
            args_len = req.args.len(),
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let artifacts_dir = req
            .artifacts_dir
            .as_deref()
            .map(|p| resolve_path_from_base(&cwd, p))
            .unwrap_or_else(|| renderdog::default_artifacts_dir(&cwd));

        std::fs::create_dir_all(&artifacts_dir)
            .map_err(|e| format!("create artifacts_dir failed: {e}"))?;

        let capture_file_template = req
            .capture_template_name
            .as_deref()
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        let request = renderdog::CaptureLaunchRequest {
            executable: resolve_path_from_base(&cwd, &req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(|p| resolve_path_from_base(&cwd, &p)),
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
            capture_file_template: capture_file_template.map(|p| p.display().to_string()),
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
        tracing::info!(
            tool = tool,
            capture_path = %req.capture_path,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);
        let output_path = resolve_path_from_base(&cwd, &req.output_path);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create output dir failed: {e}"))?;
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
        tracing::info!(
            tool = tool,
            capture_path = %req.capture_path,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
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
        Parameters(req): Parameters<TriggerCaptureRequest>,
    ) -> Result<Json<renderdog::TriggerCaptureResponse>, String> {
        let tool = "renderdoc_trigger_capture";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            target_ident = req.target_ident,
            num_frames = req.num_frames,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;

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

    #[tool(
        name = "renderdoc_export_actions_jsonl",
        description = "Export an action/event tree from a .rdc capture into a searchable JSONL format via `qrenderdoc --python`."
    )]
    async fn export_actions_jsonl(
        &self,
        Parameters(req): Parameters<ExportActionsRequest>,
    ) -> Result<Json<renderdog::ExportActionsResponse>, String> {
        let tool = "renderdoc_export_actions_jsonl";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "export actions",
            install.export_actions_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
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
        Parameters(req): Parameters<ExportBindingsIndexRequest>,
    ) -> Result<Json<renderdog::ExportBindingsIndexResponse>, String> {
        let tool = "renderdoc_export_bindings_index_jsonl";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "export bindings index",
            install.export_bindings_index_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
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
        Parameters(req): Parameters<ExportBundleRequest>,
    ) -> Result<Json<ExportBundleResponse>, String> {
        let tool = "renderdoc_export_bundle_jsonl";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let bundle = tool_result(
            tool,
            "export bundle",
            install.export_bundle_jsonl(&cwd, &req.inner),
        )?;

        let mut thumbnail_output_path: Option<String> = None;
        if req.save_thumbnail {
            let thumb_path = req
                .thumbnail_output_path
                .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
                .unwrap_or_else(|| default_thumbnail_output_path(&bundle.actions_jsonl_path));
            if let Some(parent) = Path::new(&thumb_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create thumbnail output dir failed: {e}"))?;
            }
            tool_result(
                tool,
                "save thumbnail",
                install.save_thumbnail(Path::new(&bundle.capture_path), Path::new(&thumb_path)),
            )?;
            thumbnail_output_path = Some(thumb_path);
        }

        let mut ui_pid: Option<u32> = None;
        if req.open_capture_ui {
            let child = tool_result(
                tool,
                "open capture UI",
                install.open_capture_in_ui(Path::new(&bundle.capture_path)),
            )?;
            ui_pid = Some(child.id());
        }

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            capture_path = %bundle.capture_path,
            actions_jsonl_path = %bundle.actions_jsonl_path,
            bindings_jsonl_path = %bundle.bindings_jsonl_path,
            total_actions = bundle.total_actions,
            total_drawcalls = bundle.total_drawcalls,
            "ok"
        );

        Ok(Json(ExportBundleResponse {
            bundle,
            thumbnail_output_path,
            ui_pid,
        }))
    }

    #[tool(
        name = "renderdoc_find_events",
        description = "Find matching action events (event_id + marker_path) in a .rdc capture via `qrenderdoc --python`. Useful for quickly locating event IDs for later replay tools."
    )]
    async fn find_events(
        &self,
        Parameters(req): Parameters<FindEventsRequest>,
    ) -> Result<Json<renderdog::FindEventsResponse>, String> {
        let tool = "renderdoc_find_events";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(tool, "find events", install.find_events(&cwd, &req.inner))?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            total_matches = res.total_matches,
            truncated = res.truncated,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_find_events_and_save_outputs_png",
        description = "One-shot helper: find events by marker/name and save pipeline output textures (color RTs + optional depth) to PNG."
    )]
    async fn find_events_and_save_outputs_png(
        &self,
        Parameters(req): Parameters<FindEventsAndSaveOutputsPngRequest>,
    ) -> Result<Json<FindEventsAndSaveOutputsPngResponse>, String> {
        let tool = "renderdoc_find_events_and_save_outputs_png";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture.capture_path,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let res = tool_result(
            tool,
            "find events and save outputs PNG",
            install.find_events_and_save_outputs_png(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            selected_event_id = res.selected_event_id,
            images = res.replay.outputs.len(),
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_list_textures",
        description = "List textures in a .rdc capture via `qrenderdoc --python` replay (headless)."
    )]
    async fn replay_list_textures(
        &self,
        Parameters(req): Parameters<ReplayListTexturesRequest>,
    ) -> Result<Json<renderdog::ReplayListTexturesResponse>, String> {
        let tool = "renderdoc_replay_list_textures";
        let start = Instant::now();
        tracing::info!(tool = tool, capture_path = %req.inner.capture_path, "start");
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay list textures",
            install.replay_list_textures(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            textures = res.textures.len(),
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_pick_pixel",
        description = "Pick a pixel from a texture in a .rdc capture via `qrenderdoc --python` replay."
    )]
    async fn replay_pick_pixel(
        &self,
        Parameters(req): Parameters<ReplayPickPixelRequest>,
    ) -> Result<Json<renderdog::ReplayPickPixelResponse>, String> {
        let tool = "renderdoc_replay_pick_pixel";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture_path,
            texture_index = req.inner.texture_index,
            x = req.inner.x,
            y = req.inner.y,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay pick pixel",
            install.replay_pick_pixel(&cwd, &req.inner),
        )?;

        tracing::info!(tool = tool, elapsed_ms = start.elapsed().as_millis(), "ok");
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_replay_save_texture_png",
        description = "Save a texture to PNG from a .rdc capture via `qrenderdoc --python` replay."
    )]
    async fn replay_save_texture_png(
        &self,
        Parameters(req): Parameters<ReplaySaveTexturePngRequest>,
    ) -> Result<Json<renderdog::ReplaySaveTexturePngResponse>, String> {
        let tool = "renderdoc_replay_save_texture_png";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture_path,
            texture_index = req.inner.texture_index,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay save texture PNG",
            install.replay_save_texture_png(&cwd, &req.inner),
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
        name = "renderdoc_replay_save_outputs_png",
        description = "Save current pipeline output textures (color RTs + optional depth) to PNG via `qrenderdoc --python` replay (headless)."
    )]
    async fn replay_save_outputs_png(
        &self,
        Parameters(req): Parameters<ReplaySaveOutputsPngRequest>,
    ) -> Result<Json<renderdog::ReplaySaveOutputsPngResponse>, String> {
        let tool = "renderdoc_replay_save_outputs_png";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            capture_path = %req.inner.capture_path,
            event_id = req.inner.event_id.unwrap_or(0),
            include_depth = req.inner.include_depth,
            "start"
        );
        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "replay save outputs PNG",
            install.replay_save_outputs_png(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            images = res.outputs.len(),
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_capture_and_export_actions_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export <basename>.actions.jsonl and <basename>.summary.json."
    )]
    async fn capture_and_export_actions_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportActionsRequest>,
    ) -> Result<Json<CaptureAndExportActionsResponse>, String> {
        let tool = "renderdoc_capture_and_export_actions_jsonl";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            executable = %req.inner.target.executable,
            args_len = req.inner.target.args.len(),
            "start"
        );

        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "one-shot capture/export actions",
            install.capture_and_export_actions_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = res.target_ident,
            capture_path = %res.capture_path,
            actions_jsonl_path = %res.actions_jsonl_path,
            total_actions = res.total_actions,
            "ok"
        );

        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_capture_and_export_bindings_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export <basename>.bindings.jsonl and <basename>.bindings_summary.json."
    )]
    async fn capture_and_export_bindings_index_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportBindingsIndexRequest>,
    ) -> Result<Json<CaptureAndExportBindingsIndexResponse>, String> {
        let tool = "renderdoc_capture_and_export_bindings_jsonl";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            executable = %req.inner.target.executable,
            args_len = req.inner.target.args.len(),
            "start"
        );

        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "one-shot capture/export bindings",
            install.capture_and_export_bindings_index_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = res.target_ident,
            capture_path = %res.capture_path,
            bindings_jsonl_path = %res.bindings_jsonl_path,
            total_drawcalls = res.total_drawcalls,
            "ok"
        );

        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_capture_and_export_bundle_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export <basename>.actions.jsonl (+ summary) and <basename>.bindings.jsonl (+ bindings_summary)."
    )]
    async fn capture_and_export_bundle_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportBundleRequest>,
    ) -> Result<Json<CaptureAndExportBundleResponse>, String> {
        let tool = "renderdoc_capture_and_export_bundle_jsonl";
        let start = Instant::now();
        tracing::info!(
            tool = tool,
            executable = %req.inner.target.executable,
            args_len = req.inner.target.args.len(),
            "start"
        );

        let install = require_installation(tool)?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let res = tool_result(
            tool,
            "one-shot capture/export bundle",
            install.capture_and_export_bundle_jsonl(&cwd, &req.inner),
        )?;

        tracing::info!(
            tool = tool,
            elapsed_ms = start.elapsed().as_millis(),
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
