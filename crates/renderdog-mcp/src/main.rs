use std::{
    ffi::OsString,
    io::IsTerminal,
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
    #[serde(default)]
    cwd: Option<String>,
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
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    output_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct SaveThumbnailResponse {
    output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct OpenCaptureUiRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct OpenCaptureUiResponse {
    capture_path: String,
    pid: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplayListTexturesRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    #[serde(default)]
    event_id: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplayPickPixelRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    #[serde(default)]
    event_id: Option<u32>,
    texture_index: u32,
    x: u32,
    y: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaySaveTexturePngRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    #[serde(default)]
    event_id: Option<u32>,
    texture_index: u32,
    output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaySaveOutputsPngRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    #[serde(default)]
    event_id: Option<u32>,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    basename: Option<String>,
    #[serde(default)]
    include_depth: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CaptureAndExportActionsRequest {
    #[serde(default)]
    cwd: Option<String>,
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

#[derive(Debug, Deserialize, JsonSchema)]
struct CaptureAndExportBindingsIndexRequest {
    #[serde(default)]
    cwd: Option<String>,
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
    #[serde(default)]
    include_cbuffers: bool,
    #[serde(default)]
    include_outputs: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
struct CaptureAndExportBindingsIndexResponse {
    target_ident: u32,
    capture_path: String,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,

    bindings_jsonl_path: String,
    summary_json_path: String,
    total_drawcalls: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CaptureAndExportBundleRequest {
    #[serde(default)]
    cwd: Option<String>,
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

    #[serde(default)]
    include_cbuffers: bool,
    #[serde(default)]
    include_outputs: bool,

    #[serde(default)]
    save_thumbnail: bool,
    #[serde(default)]
    thumbnail_output_path: Option<String>,
    #[serde(default)]
    open_capture_ui: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
struct CaptureAndExportBundleResponse {
    target_ident: u32,
    capture_path: String,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,

    actions_jsonl_path: String,
    actions_summary_json_path: String,
    total_actions: u64,
    drawcall_actions: u64,

    bindings_jsonl_path: String,
    bindings_summary_json_path: String,
    total_drawcalls: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui_pid: Option<u32>,
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
    #[serde(default)]
    cwd: Option<String>,
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

fn default_true() -> bool {
    true
}

fn default_max_results() -> Option<u32> {
    Some(200)
}

fn resolve_base_cwd(cwd: Option<String>) -> Result<PathBuf, String> {
    let current = std::env::current_dir().map_err(|e| format!("get cwd failed: {e}"))?;
    let Some(cwd) = cwd else {
        return Ok(current);
    };

    let p = PathBuf::from(cwd);
    if p.is_absolute() {
        Ok(p)
    } else {
        Ok(current.join(p))
    }
}

fn resolve_path_from_base(base: &Path, value: &str) -> PathBuf {
    let p = PathBuf::from(value);
    if p.is_absolute() { p } else { base.join(p) }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ExportActionsRequest {
    #[serde(default)]
    cwd: Option<String>,
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

#[derive(Debug, Deserialize, JsonSchema)]
struct ExportBindingsIndexRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    basename: Option<String>,
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
    #[serde(default)]
    include_cbuffers: bool,
    #[serde(default)]
    include_outputs: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ExportBundleRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    basename: Option<String>,

    #[serde(default)]
    save_thumbnail: bool,
    #[serde(default)]
    thumbnail_output_path: Option<String>,
    #[serde(default)]
    open_capture_ui: bool,

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

    #[serde(default)]
    include_cbuffers: bool,
    #[serde(default)]
    include_outputs: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ExportBundleResponse {
    bundle: renderdog::ExportBundleResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui_pid: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FindEventsRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,
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
    #[serde(default = "default_max_results")]
    max_results: Option<u32>,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum FindEventSelection {
    First,
    #[default]
    Last,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FindEventsAndSaveOutputsPngRequest {
    #[serde(default)]
    cwd: Option<String>,
    capture_path: String,

    #[serde(default)]
    selection: FindEventSelection,

    #[serde(default = "default_true")]
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
    #[serde(default = "default_max_results")]
    max_results: Option<u32>,

    #[serde(default)]
    output_dir: Option<String>,
    #[serde(default)]
    basename: Option<String>,
    #[serde(default)]
    include_depth: bool,
}

#[derive(Debug, Serialize, JsonSchema)]
struct FindEventsAndSaveOutputsPngResponse {
    find: renderdog::FindEventsResponse,
    selected_event_id: u32,
    replay: renderdog::ReplaySaveOutputsPngResponse,
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

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);
        let output_path = resolve_path_from_base(&cwd, &req.output_path);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create output dir failed: {e}"))?;
        }

        install
            .save_thumbnail(&capture_path, &output_path)
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

        let cwd = resolve_base_cwd(req.cwd.clone())?;

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

        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
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
        name = "renderdoc_export_bindings_index_jsonl",
        description = "Export a capture (.rdc) into a searchable bindings index: <basename>.bindings.jsonl and <basename>.bindings_summary.json."
    )]
    async fn export_bindings_index_jsonl(
        &self,
        Parameters(req): Parameters<ExportBindingsIndexRequest>,
    ) -> Result<Json<renderdog::ExportBindingsIndexResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_export_bindings_index_jsonl",
            capture_path = %req.capture_path,
            include_cbuffers = req.include_cbuffers,
            include_outputs = req.include_outputs,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_export_bindings_index_jsonl", "failed");
            tracing::debug!(tool = "renderdoc_export_bindings_index_jsonl", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
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
            .export_bindings_index_jsonl(
                &cwd,
                &renderdog::ExportBindingsIndexRequest {
                    capture_path: req.capture_path,
                    output_dir,
                    basename,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                    include_cbuffers: req.include_cbuffers,
                    include_outputs: req.include_outputs,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_export_bindings_index_jsonl", "failed");
                tracing::debug!(tool = "renderdoc_export_bindings_index_jsonl", err = %e, "details");
                format!("export bindings index failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_export_bindings_index_jsonl",
            elapsed_ms = start.elapsed().as_millis(),
            bindings_jsonl_path = %res.bindings_jsonl_path,
            total_drawcalls = res.total_drawcalls,
            "ok"
        );

        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_export_bundle_jsonl",
        description = "Export a capture (.rdc) into searchable artifacts: <basename>.actions.jsonl (+ summary) and <basename>.bindings.jsonl (+ bindings_summary)."
    )]
    async fn export_bundle_jsonl(
        &self,
        Parameters(req): Parameters<ExportBundleRequest>,
    ) -> Result<Json<ExportBundleResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_export_bundle_jsonl",
            capture_path = %req.capture_path,
            only_drawcalls = req.only_drawcalls,
            include_cbuffers = req.include_cbuffers,
            include_outputs = req.include_outputs,
            save_thumbnail = req.save_thumbnail,
            open_capture_ui = req.open_capture_ui,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_export_bundle_jsonl", "failed");
            tracing::debug!(tool = "renderdoc_export_bundle_jsonl", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
            .unwrap_or_else(|| renderdog::default_exports_dir(&cwd).display().to_string());

        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("create output_dir failed: {e}"))?;

        let basename = req.basename.unwrap_or_else(|| {
            capture_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("capture")
                .to_string()
        });

        let mut thumbnail_output_path: Option<String> = None;
        if req.save_thumbnail {
            let thumb_path = req
                .thumbnail_output_path
                .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
                .unwrap_or_else(|| {
                    Path::new(&output_dir)
                        .join(format!("{basename}.thumb.png"))
                        .display()
                        .to_string()
                });
            if let Some(parent) = Path::new(&thumb_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create thumbnail output dir failed: {e}"))?;
            }
            install
                .save_thumbnail(&capture_path, Path::new(&thumb_path))
                .map_err(|e| format!("save thumbnail failed: {e}"))?;
            thumbnail_output_path = Some(thumb_path);
        }

        let bundle = install
            .export_bundle_jsonl(
                &cwd,
                &renderdog::ExportBundleRequest {
                    capture_path: req.capture_path.clone(),
                    output_dir,
                    basename,
                    only_drawcalls: req.only_drawcalls,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                    include_cbuffers: req.include_cbuffers,
                    include_outputs: req.include_outputs,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_export_bundle_jsonl", "failed");
                tracing::debug!(tool = "renderdoc_export_bundle_jsonl", err = %e, "details");
                format!("export bundle failed: {e}")
            })?;

        let mut ui_pid: Option<u32> = None;
        if req.open_capture_ui {
            let child = install
                .open_capture_in_ui(&capture_path)
                .map_err(|e| format!("open capture UI failed: {e}"))?;
            ui_pid = Some(child.id());
        }

        tracing::info!(
            tool = "renderdoc_export_bundle_jsonl",
            elapsed_ms = start.elapsed().as_millis(),
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_find_events",
            capture_path = %req.capture_path,
            only_drawcalls = req.only_drawcalls,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_find_events", "failed");
            tracing::debug!(tool = "renderdoc_find_events", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let res = install
            .find_events(
                &cwd,
                &renderdog::FindEventsRequest {
                    capture_path: req.capture_path,
                    only_drawcalls: req.only_drawcalls,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                    max_results: req.max_results,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_find_events", "failed");
                tracing::debug!(tool = "renderdoc_find_events", err = %e, "details");
                format!("find events failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_find_events",
            elapsed_ms = start.elapsed().as_millis(),
            matches = res.matches.len(),
            truncated = res.truncated,
            "ok"
        );
        Ok(Json(res))
    }

    #[tool(
        name = "renderdoc_find_events_and_save_outputs_png",
        description = "One-shot helper: find matching events (by marker/name filters) and save current pipeline outputs to PNG at the selected event via headless replay."
    )]
    async fn find_events_and_save_outputs_png(
        &self,
        Parameters(req): Parameters<FindEventsAndSaveOutputsPngRequest>,
    ) -> Result<Json<FindEventsAndSaveOutputsPngResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_find_events_and_save_outputs_png",
            capture_path = %req.capture_path,
            only_drawcalls = req.only_drawcalls,
            include_depth = req.include_depth,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(
                tool = "renderdoc_find_events_and_save_outputs_png",
                "failed"
            );
            tracing::debug!(
                tool = "renderdoc_find_events_and_save_outputs_png",
                err = %e,
                "details"
            );
            format!("detect installation failed: {e}")
        })?;

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);

        let find = install
            .find_events(
                &cwd,
                &renderdog::FindEventsRequest {
                    capture_path: capture_path.display().to_string(),
                    only_drawcalls: req.only_drawcalls,
                    marker_prefix: req.marker_prefix.clone(),
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains.clone(),
                    marker_contains: req.marker_contains.clone(),
                    case_sensitive: req.case_sensitive,
                    max_results: req.max_results,
                },
            )
            .map_err(|e| format!("find events failed: {e}"))?;

        if find.total_matches == 0 {
            return Err(
                "no matching events found; refine filters or disable only_drawcalls".into(),
            );
        }

        let selected_event_id = match req.selection {
            FindEventSelection::First => find
                .first_event_id
                .or_else(|| find.matches.first().map(|m| m.event_id))
                .ok_or_else(|| "no matching events found".to_string())?,
            FindEventSelection::Last => find
                .last_event_id
                .or_else(|| find.matches.last().map(|m| m.event_id))
                .ok_or_else(|| "no matching events found".to_string())?,
        };

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
            .unwrap_or_else(|| {
                renderdog::default_exports_dir(&cwd)
                    .join("replay")
                    .display()
                    .to_string()
            });
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| format!("create output_dir failed: {e}"))?;

        let basename = req.basename.unwrap_or_else(|| {
            capture_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("capture")
                .to_string()
        });

        let replay = install
            .replay_save_outputs_png(
                &cwd,
                &renderdog::ReplaySaveOutputsPngRequest {
                    capture_path: capture_path.display().to_string(),
                    event_id: Some(selected_event_id),
                    output_dir,
                    basename,
                    include_depth: req.include_depth,
                },
            )
            .map_err(|e| format!("replay save outputs failed: {e}"))?;

        tracing::info!(
            tool = "renderdoc_find_events_and_save_outputs_png",
            elapsed_ms = start.elapsed().as_millis(),
            selected_event_id,
            outputs = replay.outputs.len(),
            "ok"
        );

        Ok(Json(FindEventsAndSaveOutputsPngResponse {
            find,
            selected_event_id,
            replay,
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

        let cwd = resolve_base_cwd(req.cwd.clone())?;
        let capture_path = resolve_path_from_base(&cwd, &req.capture_path);

        let child = install.open_capture_in_ui(&capture_path).map_err(|e| {
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
            capture_path: capture_path.display().to_string(),
            pid,
        }))
    }

    #[tool(
        name = "renderdoc_replay_list_textures",
        description = "List textures in a .rdc capture via `qrenderdoc --python` replay (headless)."
    )]
    async fn replay_list_textures(
        &self,
        Parameters(req): Parameters<ReplayListTexturesRequest>,
    ) -> Result<Json<renderdog::ReplayListTexturesResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_replay_list_textures",
            capture_path = %req.capture_path,
            event_id = req.event_id,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_replay_list_textures", "failed");
            tracing::debug!(tool = "renderdoc_replay_list_textures", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;
        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let res = install
            .replay_list_textures(
                &cwd,
                &renderdog::ReplayListTexturesRequest {
                    capture_path: req.capture_path,
                    event_id: req.event_id,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_replay_list_textures", "failed");
                tracing::debug!(tool = "renderdoc_replay_list_textures", err = %e, "details");
                format!("replay list textures failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_replay_list_textures",
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_replay_pick_pixel",
            capture_path = %req.capture_path,
            event_id = req.event_id,
            texture_index = req.texture_index,
            x = req.x,
            y = req.y,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_replay_pick_pixel", "failed");
            tracing::debug!(tool = "renderdoc_replay_pick_pixel", err = %e, "details");
            format!("detect installation failed: {e}")
        })?;
        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let res = install
            .replay_pick_pixel(
                &cwd,
                &renderdog::ReplayPickPixelRequest {
                    capture_path: req.capture_path,
                    event_id: req.event_id,
                    texture_index: req.texture_index,
                    x: req.x,
                    y: req.y,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_replay_pick_pixel", "failed");
                tracing::debug!(tool = "renderdoc_replay_pick_pixel", err = %e, "details");
                format!("replay pick pixel failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_replay_pick_pixel",
            elapsed_ms = start.elapsed().as_millis(),
            "ok"
        );
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_replay_save_texture_png",
            capture_path = %req.capture_path,
            event_id = req.event_id,
            texture_index = req.texture_index,
            output_path = %req.output_path,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_replay_save_texture_png", "failed");
            tracing::debug!(
                tool = "renderdoc_replay_save_texture_png",
                err = %e,
                "details"
            );
            format!("detect installation failed: {e}")
        })?;
        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let res = install
            .replay_save_texture_png(
                &cwd,
                &renderdog::ReplaySaveTexturePngRequest {
                    capture_path: req.capture_path,
                    event_id: req.event_id,
                    texture_index: req.texture_index,
                    output_path: req.output_path,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_replay_save_texture_png", "failed");
                tracing::debug!(
                    tool = "renderdoc_replay_save_texture_png",
                    err = %e,
                    "details"
                );
                format!("replay save texture failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_replay_save_texture_png",
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
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_replay_save_outputs_png",
            capture_path = %req.capture_path,
            event_id = req.event_id,
            include_depth = req.include_depth,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_replay_save_outputs_png", "failed");
            tracing::debug!(
                tool = "renderdoc_replay_save_outputs_png",
                err = %e,
                "details"
            );
            format!("detect installation failed: {e}")
        })?;
        let cwd = resolve_base_cwd(req.cwd.clone())?;

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
            .unwrap_or_else(|| {
                renderdog::default_exports_dir(&cwd)
                    .join("replay")
                    .display()
                    .to_string()
            });
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
            .replay_save_outputs_png(
                &cwd,
                &renderdog::ReplaySaveOutputsPngRequest {
                    capture_path: req.capture_path,
                    event_id: req.event_id,
                    output_dir,
                    basename,
                    include_depth: req.include_depth,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_replay_save_outputs_png", "failed");
                tracing::debug!(
                    tool = "renderdoc_replay_save_outputs_png",
                    err = %e,
                    "details"
                );
                format!("replay save outputs failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_replay_save_outputs_png",
            elapsed_ms = start.elapsed().as_millis(),
            outputs = res.outputs.len(),
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

        let launch_req = renderdog::CaptureLaunchRequest {
            executable: resolve_path_from_base(&cwd, &req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(|p| resolve_path_from_base(&cwd, &p)),
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
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
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

    #[tool(
        name = "renderdoc_capture_and_export_bindings_index_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export <basename>.bindings.jsonl and <basename>.bindings_summary.json."
    )]
    async fn capture_and_export_bindings_index_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportBindingsIndexRequest>,
    ) -> Result<Json<CaptureAndExportBindingsIndexResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_capture_and_export_bindings_index_jsonl",
            executable = %req.executable,
            args_len = req.args.len(),
            include_cbuffers = req.include_cbuffers,
            include_outputs = req.include_outputs,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(
                tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                "failed"
            );
            tracing::debug!(
                tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                err = %e,
                "details"
            );
            format!("detect installation failed: {e}")
        })?;

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

        let launch_req = renderdog::CaptureLaunchRequest {
            executable: resolve_path_from_base(&cwd, &req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(|p| resolve_path_from_base(&cwd, &p)),
            capture_file_template: capture_file_template.clone(),
        };

        let launch_res = install.launch_capture(&launch_req).map_err(|e| {
            tracing::error!(
                tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                "failed"
            );
            tracing::debug!(
                tool = "renderdoc_capture_and_export_bindings_index_jsonl",
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
                    tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                    "failed"
                );
                tracing::debug!(
                    tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                    err = %e,
                    "details"
                );
                format!("trigger capture failed: {e}")
            })?;

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
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
            .export_bindings_index_jsonl(
                &cwd,
                &renderdog::ExportBindingsIndexRequest {
                    capture_path: capture_res.capture_path.clone(),
                    output_dir,
                    basename,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                    include_cbuffers: req.include_cbuffers,
                    include_outputs: req.include_outputs,
                },
            )
            .map_err(|e| {
                tracing::error!(
                    tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                    "failed"
                );
                tracing::debug!(
                    tool = "renderdoc_capture_and_export_bindings_index_jsonl",
                    err = %e,
                    "details"
                );
                format!("export bindings index failed: {e}")
            })?;

        tracing::info!(
            tool = "renderdoc_capture_and_export_bindings_index_jsonl",
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = launch_res.target_ident,
            capture_path = %export_res.capture_path,
            bindings_jsonl_path = %export_res.bindings_jsonl_path,
            total_drawcalls = export_res.total_drawcalls,
            "ok"
        );

        Ok(Json(CaptureAndExportBindingsIndexResponse {
            target_ident: launch_res.target_ident,
            capture_path: export_res.capture_path,
            capture_file_template: capture_file_template.map(|p| p.display().to_string()),
            stdout: launch_res.stdout,
            stderr: launch_res.stderr,
            bindings_jsonl_path: export_res.bindings_jsonl_path,
            summary_json_path: export_res.summary_json_path,
            total_drawcalls: export_res.total_drawcalls,
        }))
    }

    #[tool(
        name = "renderdoc_capture_and_export_bundle_jsonl",
        description = "One-shot workflow: launch target under renderdoccmd capture, trigger capture via target control, then export both <basename>.actions.jsonl (+ summary) and <basename>.bindings.jsonl (+ bindings_summary)."
    )]
    async fn capture_and_export_bundle_jsonl(
        &self,
        Parameters(req): Parameters<CaptureAndExportBundleRequest>,
    ) -> Result<Json<CaptureAndExportBundleResponse>, String> {
        let start = Instant::now();
        tracing::info!(
            tool = "renderdoc_capture_and_export_bundle_jsonl",
            executable = %req.executable,
            args_len = req.args.len(),
            only_drawcalls = req.only_drawcalls,
            include_cbuffers = req.include_cbuffers,
            include_outputs = req.include_outputs,
            save_thumbnail = req.save_thumbnail,
            open_capture_ui = req.open_capture_ui,
            "start"
        );

        let install = renderdog::RenderDocInstallation::detect().map_err(|e| {
            tracing::error!(tool = "renderdoc_capture_and_export_bundle_jsonl", "failed");
            tracing::debug!(
                tool = "renderdoc_capture_and_export_bundle_jsonl",
                err = %e,
                "details"
            );
            format!("detect installation failed: {e}")
        })?;

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

        let launch_req = renderdog::CaptureLaunchRequest {
            executable: resolve_path_from_base(&cwd, &req.executable),
            args: req.args.into_iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(|p| resolve_path_from_base(&cwd, &p)),
            capture_file_template: capture_file_template.clone(),
        };

        let launch_res = install.launch_capture(&launch_req).map_err(|e| {
            tracing::error!(tool = "renderdoc_capture_and_export_bundle_jsonl", "failed");
            tracing::debug!(
                tool = "renderdoc_capture_and_export_bundle_jsonl",
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
                tracing::error!(tool = "renderdoc_capture_and_export_bundle_jsonl", "failed");
                tracing::debug!(
                    tool = "renderdoc_capture_and_export_bundle_jsonl",
                    err = %e,
                    "details"
                );
                format!("trigger capture failed: {e}")
            })?;

        let output_dir = req
            .output_dir
            .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
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
            .export_bundle_jsonl(
                &cwd,
                &renderdog::ExportBundleRequest {
                    capture_path: capture_res.capture_path.clone(),
                    output_dir: output_dir.clone(),
                    basename: basename.clone(),
                    only_drawcalls: req.only_drawcalls,
                    marker_prefix: req.marker_prefix,
                    event_id_min: req.event_id_min,
                    event_id_max: req.event_id_max,
                    name_contains: req.name_contains,
                    marker_contains: req.marker_contains,
                    case_sensitive: req.case_sensitive,
                    include_cbuffers: req.include_cbuffers,
                    include_outputs: req.include_outputs,
                },
            )
            .map_err(|e| {
                tracing::error!(tool = "renderdoc_capture_and_export_bundle_jsonl", "failed");
                tracing::debug!(
                    tool = "renderdoc_capture_and_export_bundle_jsonl",
                    err = %e,
                    "details"
                );
                format!("export bundle failed: {e}")
            })?;

        let mut thumbnail_output_path: Option<String> = None;
        if req.save_thumbnail {
            let thumb_path = req
                .thumbnail_output_path
                .map(|p| resolve_path_from_base(&cwd, &p).display().to_string())
                .unwrap_or_else(|| {
                    Path::new(&output_dir)
                        .join(format!("{basename}.thumb.png"))
                        .display()
                        .to_string()
                });
            if let Some(parent) = Path::new(&thumb_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create thumbnail output dir failed: {e}"))?;
            }
            install
                .save_thumbnail(Path::new(&export_res.capture_path), Path::new(&thumb_path))
                .map_err(|e| format!("save thumbnail failed: {e}"))?;
            thumbnail_output_path = Some(thumb_path);
        }

        let mut ui_pid: Option<u32> = None;
        if req.open_capture_ui {
            let child = install
                .open_capture_in_ui(Path::new(&export_res.capture_path))
                .map_err(|e| format!("open capture UI failed: {e}"))?;
            ui_pid = Some(child.id());
        }

        tracing::info!(
            tool = "renderdoc_capture_and_export_bundle_jsonl",
            elapsed_ms = start.elapsed().as_millis(),
            target_ident = launch_res.target_ident,
            capture_path = %export_res.capture_path,
            actions_jsonl_path = %export_res.actions_jsonl_path,
            bindings_jsonl_path = %export_res.bindings_jsonl_path,
            total_actions = export_res.total_actions,
            total_drawcalls = export_res.total_drawcalls,
            "ok"
        );

        Ok(Json(CaptureAndExportBundleResponse {
            target_ident: launch_res.target_ident,
            capture_path: export_res.capture_path,
            capture_file_template: capture_file_template.map(|p| p.display().to_string()),
            stdout: launch_res.stdout,
            stderr: launch_res.stderr,

            actions_jsonl_path: export_res.actions_jsonl_path,
            actions_summary_json_path: export_res.actions_summary_json_path,
            total_actions: export_res.total_actions,
            drawcall_actions: export_res.drawcall_actions,

            bindings_jsonl_path: export_res.bindings_jsonl_path,
            bindings_summary_json_path: export_res.bindings_summary_json_path,
            total_drawcalls: export_res.total_drawcalls,

            thumbnail_output_path,
            ui_pid,
        }))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    if std::io::stdin().is_terminal() {
        eprintln!(
            "renderdog-mcp is an MCP stdio server.\n\
It is meant to be launched by an MCP client (Claude Code / Codex / Gemini CLI).\n\
See the workspace README for setup: https://github.com/Latias94/renderdog\n"
        );
    }

    let server = RenderdogMcpServer::new();
    let service = match server.serve(stdio()).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "renderdog-mcp failed to start. If you ran it directly, make sure an MCP client is launching it.\n\
Error: {e}"
            );
            return Err(e.into());
        }
    };

    if let Err(e) = service.waiting().await {
        eprintln!(
            "renderdog-mcp stopped. If you ran it directly, this usually means stdin was closed.\n\
Launch it via an MCP client (stdio transport).\n\
Error: {e}"
        );
        return Err(e.into());
    }
    Ok(())
}
