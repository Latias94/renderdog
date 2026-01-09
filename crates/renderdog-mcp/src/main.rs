use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::Instant,
};

use rmcp::{
    Json, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use renderdog_automation as renderdog;

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_target(false).init();
}

#[derive(Debug, Serialize, JsonSchema)]
struct DetectInstallationResponse {
    root_dir: String,
    qrenderdoc_exe: String,
    renderdoccmd_exe: String,
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vulkan_layer: Option<renderdog::VulkanLayerDiagnosis>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct LaunchCaptureRequest {
    executable: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    working_dir: Option<String>,
    #[serde(default)]
    artifacts_dir: Option<String>,
    #[serde(default)]
    capture_template_name: Option<String>,
}

#[derive(Debug, Serialize, JsonSchema)]
struct LaunchCaptureResponse {
    target_ident: u32,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SaveThumbnailRequest {
    capture_path: String,
    output_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct SaveThumbnailResponse {
    output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct OpenCaptureUiRequest {
    capture_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct OpenCaptureUiResponse {
    capture_path: String,
    pid: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CaptureAndExportActionsRequest {
    executable: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    working_dir: Option<String>,
    #[serde(default)]
    artifacts_dir: Option<String>,
    #[serde(default)]
    capture_template_name: Option<String>,

    #[serde(default = "default_host")]
    host: String,
    #[serde(default = "default_frames")]
    num_frames: u32,
    #[serde(default = "default_timeout_s")]
    timeout_s: u32,

    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    basename: Option<String>,
    #[serde(default)]
    only_drawcalls: bool,
    #[serde(default)]
    marker_prefix: Option<String>,
    #[serde(default)]
    event_id_min: Option<u32>,
    #[serde(default)]
    event_id_max: Option<u32>,
    #[serde(default)]
    name_contains: Option<String>,
    #[serde(default)]
    marker_contains: Option<String>,
    #[serde(default)]
    case_sensitive: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
struct CaptureAndExportActionsResponse {
    target_ident: u32,
    capture_path: String,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,

    actions_jsonl_path: String,
    summary_json_path: String,
    total_actions: u64,
    drawcall_actions: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TriggerCaptureRequest {
    #[serde(default = "default_host")]
    host: String,
    target_ident: u32,
    #[serde(default = "default_frames")]
    num_frames: u32,
    #[serde(default = "default_timeout_s")]
    timeout_s: u32,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_frames() -> u32 {
    1
}

fn default_timeout_s() -> u32 {
    60
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ExportActionsRequest {
    capture_path: String,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    basename: Option<String>,
    #[serde(default)]
    only_drawcalls: bool,
    #[serde(default)]
    marker_prefix: Option<String>,
    #[serde(default)]
    event_id_min: Option<u32>,
    #[serde(default)]
    event_id_max: Option<u32>,
    #[serde(default)]
    name_contains: Option<String>,
    #[serde(default)]
    marker_contains: Option<String>,
    #[serde(default)]
    case_sensitive: bool,
}

#[derive(Clone)]
struct RenderdogMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl rmcp::ServerHandler for RenderdogMcpServer {}

#[tool_router(router = tool_router)]
impl RenderdogMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "renderdoc_detect_installation",
        description = "Detect local RenderDoc installation and return tool paths."
    )]
    async fn detect_installation(&self) -> Result<Json<DetectInstallationResponse>, String> {
        let start = Instant::now();
        tracing::info!(tool = "renderdoc_detect_installation", "start");
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_detect_installation", "failed");
            tracing::debug!(tool = "renderdoc_detect_installation", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let version = install.version().ok().map(|s| s.trim().to_string());
        let vulkan_layer = install.diagnose_vulkan_layer().ok();

        tracing::info!(
            tool = "renderdoc_detect_installation",
            elapsed_ms = start.elapsed().as_millis(),
            "ok"
        );
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
        let start = Instant::now();
        tracing::info!(tool = "renderdoc_vulkanlayer_diagnose", "start");
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_vulkanlayer_diagnose", "failed");
            tracing::debug!(tool = "renderdoc_vulkanlayer_diagnose", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;
        let diag = install.diagnose_vulkan_layer().map_err(|e| {
            tracing::error!(tool = "renderdoc_vulkanlayer_diagnose", "failed");
            tracing::debug!(tool = "renderdoc_vulkanlayer_diagnose", err = %e, "details");
            format!("diagnose vulkan layer failed: {e}")
        })?;
        tracing::info!(
            tool = "renderdoc_vulkanlayer_diagnose",
            elapsed_ms = start.elapsed().as_millis(),
            "ok"
        );
        Ok(Json(diag))
    }

    #[tool(
        name = "renderdoc_diagnose_environment",
        description = "Diagnose RenderDoc environment (paths, renderdoccmd version, Vulkan layer registration, and key Vulkan-related env vars) and return warnings + suggested fixes."
    )]
    async fn diagnose_environment(&self) -> Result<Json<renderdog::EnvironmentDiagnosis>, String> {
        let start = Instant::now();
        tracing::info!(tool = "renderdoc_diagnose_environment", "start");
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_diagnose_environment", "failed");
            tracing::debug!(tool = "renderdoc_diagnose_environment", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;
        let diag = install.diagnose_environment().map_err(|e| {
            tracing::error!(tool = "renderdoc_diagnose_environment", "failed");
            tracing::debug!(tool = "renderdoc_diagnose_environment", err = %e, "details");
            format!("diagnose environment failed: {e}")
        })?;
        tracing::info!(
            tool = "renderdoc_diagnose_environment",
            elapsed_ms = start.elapsed().as_millis(),
            "ok"
        );
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_launch_capture",
            executable = %req.executable,
            args_len = req.args.len(),
            "start"
        );
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_launch_capture", "failed");
            tracing::debug!(tool = "renderdoc_launch_capture", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let cwd = std::env::current_dir().map_err(|e| format!("get cwd failed: {e}"))?;

        let artifacts_dir = req
            .artifacts_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| renderdog::default_artifacts_dir(&cwd));

        std::fs::create_dir_all(&artifacts_dir)
            .map_err(|e| format!("create artifacts_dir failed: {e}"))?;

        let capture_file_template = req
            .capture_template_name
            .as_deref()
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        let request = renderdog::CaptureLaunchRequest {
            executable: PathBuf::from(req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(PathBuf::from),
            capture_file_template: capture_file_template.clone(),
        };

        let res = install.launch_capture(&request).map_err(|e| {
            tracing::error!(tool = "renderdoc_launch_capture", "failed");
            tracing::debug!(tool = "renderdoc_launch_capture", err = %e, "details");
            format!("launch capture failed: {e}")
        })?;

        tracing::info!(
            tool = "renderdoc_launch_capture",
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_save_thumbnail",
            capture_path = %req.capture_path,
            output_path = %req.output_path,
            "start"
        );
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_save_thumbnail", "failed");
            tracing::debug!(tool = "renderdoc_save_thumbnail", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let capture_path = Path::new(&req.capture_path);
        let output_path = Path::new(&req.output_path);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create output dir failed: {e}"))?;
        }

        install
            .save_thumbnail(capture_path, output_path)
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_save_thumbnail", "failed");
                tracing::debug!(tool = "renderdoc_save_thumbnail", err = %e, "details");
                format!("save thumbnail failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_save_thumbnail",
            elapsed_ms = start.elapsed().as_millis(),
            "ok"
        );
        Ok(Json(SaveThumbnailResponse {
            output_path: output_path.display().to_string(),
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_trigger_capture",
            host = %req.host,
            target_ident = req.target_ident,
            frames = req.num_frames,
            timeout_s = req.timeout_s,
            "start"
        );
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_trigger_capture", "failed");
            tracing::debug!(tool = "renderdoc_trigger_capture", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let cwd = std::env::current_dir().map_err(|e| format!("get cwd failed: {e}"))?;

        let res = install
            .trigger_capture_via_target_control(
                &cwd,
                &renderdog::TriggerCaptureRequest {
                    host: req.host,
                    target_ident: req.target_ident,
                    num_frames: req.num_frames,
                    timeout_s: req.timeout_s,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_trigger_capture", "failed");
                tracing::debug!(tool = "renderdoc_trigger_capture", err = %e, "details");
                format!("trigger capture failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_trigger_capture",
            elapsed_ms = start.elapsed().as_millis(),
            capture_path = %res.capture_path,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_export_actions_jsonl",
        description = "Export a capture (.rdc) into searchable artifacts: <basename>.actions.jsonl and <basename>.summary.json."
    )]
    async fn export_actions_jsonl(
        &self,
        Parameters(req): Parameters<ExportActionsRequest>,
    ) -> Result<Json<renderdog::ExportActionsResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_export_actions_jsonl",
            capture_path = %req.capture_path,
            only_drawcalls = req.only_drawcalls,
            "start"
        );
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_export_actions_jsonl", "failed");
            tracing::debug!(tool = "renderdoc_export_actions_jsonl", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let cwd = std::env::current_dir().map_err(|e| format!("get cwd failed: {e}"))?;

        let output_dir = req
            .output_dir
            .unwrap_or_else(|| renderdog::default_exports_dir(&cwd).display().to_string());

        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("create output_dir failed: {e}"))?;

        let basename = req.basename.unwrap_or_else(|| {
            Path::new(&req.capture_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("capture")
                .to_string()
        });

        let res = install
            .export_actions_jsonl(
                &cwd,
                &renderdog::ExportActionsRequest {
                    capture_path: req.capture_path,
                    output_dir,
                    basename,
                    only_drawcalls: req.only_drawcalls,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_export_actions_jsonl", "failed");
                tracing::debug!(tool = "renderdoc_export_actions_jsonl", err = %e, "details");
                format!("export actions failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_export_actions_jsonl",
            elapsed_ms = start.elapsed().as_millis(),
            actions_jsonl_path = %res.actions_jsonl_path,
            total_actions = res.total_actions,
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_open_capture_ui",
            capture_path = %req.capture_path,
            "start"
        );
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_open_capture_ui", "failed");
            tracing::debug!(tool = "renderdoc_open_capture_ui", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let child = install
            .open_capture_in_ui(Path::new(&req.capture_path))
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_open_capture_ui", "failed");
                tracing::debug!(tool = "renderdoc_open_capture_ui", err = %e, "details");
                format!("open capture UI failed: {e}")
            })?;

        let pid = child.id();

        tracing::info!(
            tool = "renderdoc_open_capture_ui",
            elapsed_ms = start.elapsed().as_millis(),
            pid,
            "ok"
        );
        Ok(Json(OpenCaptureUiResponse {
            capture_path: req.capture_path,
            pid,
        }))
    }

    #[tool(
        name = "renderdoc_capture_and_export_actions_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export <basename>.actions.jsonl and <basename>.summary.json."
    )]
    async fn capture_and_export_actions_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportActionsRequest>,
    ) -> Result<Json<CaptureAndExportActionsResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_capture_and_export_actions_jsonl",
            executable = %req.executable,
            args_len = req.args.len(),
            only_drawcalls = req.only_drawcalls,
            "start"
        );
        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(
                tool = "renderdoc_capture_and_export_actions_jsonl",
                "failed"
            );
            tracing::debug!(
                tool = "renderdoc_capture_and_export_actions_jsonl",
                err = %e,
                "details"
            );
            format!("detect installation failed: {e}")
        })?;

        let cwd = std::env::current_dir().map_err(|e| format!("get cwd failed: {e}"))?;

        let artifacts_dir = req
            .artifacts_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(|| renderdog::default_artifacts_dir(&cwd));

        std::fs::create_dir_all(&artifacts_dir)
            .map_err(|e| format!("create artifacts_dir failed: {e}"))?;

        let capture_file_template = req
            .capture_template_name
            .as_deref()
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        let launch_req = renderdog::CaptureLaunchRequest {
            executable: PathBuf::from(req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(PathBuf::from),
            capture_file_template: capture_file_template.clone(),
        };

        let launch_res = install.launch_capture(&launch_req).map_err(|e| {
            tracing::error!(
                tool = "renderdoc_capture_and_export_actions_jsonl",
                "failed"
            );
            tracing::debug!(
                tool = "renderdoc_capture_and_export_actions_jsonl",
                err = %e,
                "details"
            );
            format!("launch capture failed: {e}")
        })?;

        let capture_res = install
            .trigger_capture_via_target_control(
                &cwd,
                &renderdog::TriggerCaptureRequest {
                    host: req.host,
                    target_ident: launch_res.target_ident,
                    num_frames: req.num_frames,
                    timeout_s: req.timeout_s,
                },
            )
            .map_err(|e| {
                tracing::error!(
                    tool = "renderdoc_capture_and_export_actions_jsonl",
                    "failed"
                );
                tracing::debug!(
                    tool = "renderdoc_capture_and_export_actions_jsonl",
                    err = %e,
                    "details"
                );
                format!("trigger capture failed: {e}")
            })?;

        let output_dir = req
            .output_dir
            .unwrap_or_else(|| renderdog::default_exports_dir(&cwd).display().to_string());

        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("create output_dir failed: {e}"))?;

        let basename = req.basename.unwrap_or_else(|| {
            Path::new(&capture_res.capture_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("capture")
                .to_string()
        });

        let export_res = install
            .export_actions_jsonl(
                &cwd,
                &renderdog::ExportActionsRequest {
                    capture_path: capture_res.capture_path.clone(),
                    output_dir,
                    basename,
                    only_drawcalls: req.only_drawcalls,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                },
            )
            .map_err(|e| {
                tracing::error!(
                    tool = "renderdoc_capture_and_export_actions_jsonl",
                    "failed"
                );
                tracing::debug!(
                    tool = "renderdoc_capture_and_export_actions_jsonl",
                    err = %e,
                    "details"
                );
                format!("export actions failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_capture_and_export_actions_jsonl",
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = launch_res.target_ident,
            capture_path = %export_res.capture_path,
            actions_jsonl_path = %export_res.actions_jsonl_path,
            total_actions = export_res.total_actions,
            "ok"
        );
        Ok(Json(CaptureAndExportActionsResponse {
            target_ident: launch_res.target_ident,
            capture_path: export_res.capture_path,
            capture_file_template: capture_file_template.map(|p| p.display().to_string()),
            stdout: launch_res.stdout,
            stderr: launch_res.stderr,
            actions_jsonl_path: export_res.actions_jsonl_path,
            summary_json_path: export_res.summary_json_path,
            total_actions: export_res.total_actions,
            drawcall_actions: export_res.drawcall_actions,
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let server = RenderdogMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
