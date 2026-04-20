use std::{ffi::OsString, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CaptureLaunchError, CaptureLaunchRequest, ExportActionsError, ExportActionsRequest,
    ExportBindingsIndexError, ExportBindingsIndexRequest, ExportBundleError, ExportBundleRequest,
    OpenCaptureUiError, RenderDocInstallation, TriggerCaptureError, TriggerCaptureRequest,
    default_artifacts_dir, default_exports_dir, resolve_path_from_cwd,
};

fn default_host() -> String {
    "localhost".to_string()
}

fn default_frames() -> u32 {
    1
}

fn default_timeout_s() -> u32 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportActionsRequest {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub artifacts_dir: Option<String>,
    #[serde(default)]
    pub capture_template_name: Option<String>,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
    #[serde(default)]
    pub only_drawcalls: bool,
    #[serde(default)]
    pub marker_prefix: Option<String>,
    #[serde(default)]
    pub event_id_min: Option<u32>,
    #[serde(default)]
    pub event_id_max: Option<u32>,
    #[serde(default)]
    pub name_contains: Option<String>,
    #[serde(default)]
    pub marker_contains: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBindingsIndexRequest {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub artifacts_dir: Option<String>,
    #[serde(default)]
    pub capture_template_name: Option<String>,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
    #[serde(default)]
    pub marker_prefix: Option<String>,
    #[serde(default)]
    pub event_id_min: Option<u32>,
    #[serde(default)]
    pub event_id_max: Option<u32>,
    #[serde(default)]
    pub name_contains: Option<String>,
    #[serde(default)]
    pub marker_contains: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub include_cbuffers: bool,
    #[serde(default)]
    pub include_outputs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBundleRequest {
    pub executable: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub working_dir: Option<String>,
    #[serde(default)]
    pub artifacts_dir: Option<String>,
    #[serde(default)]
    pub capture_template_name: Option<String>,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_frames")]
    pub num_frames: u32,
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u32,
    #[serde(default)]
    pub output_dir: Option<String>,
    #[serde(default)]
    pub basename: Option<String>,
    #[serde(default)]
    pub only_drawcalls: bool,
    #[serde(default)]
    pub marker_prefix: Option<String>,
    #[serde(default)]
    pub event_id_min: Option<u32>,
    #[serde(default)]
    pub event_id_max: Option<u32>,
    #[serde(default)]
    pub name_contains: Option<String>,
    #[serde(default)]
    pub marker_contains: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub include_cbuffers: bool,
    #[serde(default)]
    pub include_outputs: bool,
    #[serde(default)]
    pub save_thumbnail: bool,
    #[serde(default)]
    pub thumbnail_output_path: Option<String>,
    #[serde(default)]
    pub open_capture_ui: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportActionsResponse {
    pub target_ident: u32,
    pub capture_path: String,
    pub capture_file_template: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub actions_jsonl_path: String,
    pub summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBindingsIndexResponse {
    pub target_ident: u32,
    pub capture_path: String,
    pub capture_file_template: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub bindings_jsonl_path: String,
    pub summary_json_path: String,
    pub total_drawcalls: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBundleResponse {
    pub target_ident: u32,
    pub capture_path: String,
    pub capture_file_template: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub actions_jsonl_path: String,
    pub actions_summary_json_path: String,
    pub total_actions: u64,
    pub drawcall_actions: u64,
    pub bindings_jsonl_path: String,
    pub bindings_summary_json_path: String,
    pub total_drawcalls: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_pid: Option<u32>,
}

#[derive(Debug, Error)]
pub enum PrepareOneShotCaptureError {
    #[error("failed to create artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("launch capture failed: {0}")]
    Launch(#[from] CaptureLaunchError),
    #[error("trigger capture failed: {0}")]
    Trigger(#[from] TriggerCaptureError),
}

#[derive(Debug, Error)]
pub enum CaptureAndExportActionsError {
    #[error(transparent)]
    Prepare(#[from] PrepareOneShotCaptureError),
    #[error("export actions failed: {0}")]
    Export(#[from] ExportActionsError),
}

#[derive(Debug, Error)]
pub enum CaptureAndExportBindingsIndexError {
    #[error(transparent)]
    Prepare(#[from] PrepareOneShotCaptureError),
    #[error("export bindings index failed: {0}")]
    Export(#[from] ExportBindingsIndexError),
}

#[derive(Debug, Error)]
pub enum CaptureAndExportBundleError {
    #[error(transparent)]
    Prepare(#[from] PrepareOneShotCaptureError),
    #[error("export bundle failed: {0}")]
    Export(#[from] ExportBundleError),
    #[error("save thumbnail failed: {0}")]
    SaveThumbnail(std::io::Error),
    #[error("open capture UI failed: {0}")]
    OpenCaptureUi(#[from] OpenCaptureUiError),
}

struct PreparedOneShotCapture {
    target_ident: u32,
    capture_path: String,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,
    output_dir: String,
    basename: String,
}

struct OneShotCaptureRequest<'a> {
    executable: &'a str,
    args: &'a [String],
    working_dir: Option<&'a str>,
    artifacts_dir: Option<&'a str>,
    capture_template_name: Option<&'a str>,
    host: &'a str,
    num_frames: u32,
    timeout_s: u32,
    output_dir: Option<&'a str>,
    basename: Option<&'a str>,
}

impl RenderDocInstallation {
    pub fn capture_and_export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportActionsRequest,
    ) -> Result<CaptureAndExportActionsResponse, CaptureAndExportActionsError> {
        let prepared = self.prepare_one_shot_capture(
            cwd,
            OneShotCaptureRequest {
                executable: &req.executable,
                args: &req.args,
                working_dir: req.working_dir.as_deref(),
                artifacts_dir: req.artifacts_dir.as_deref(),
                capture_template_name: req.capture_template_name.as_deref(),
                host: &req.host,
                num_frames: req.num_frames,
                timeout_s: req.timeout_s,
                output_dir: req.output_dir.as_deref(),
                basename: req.basename.as_deref(),
            },
        )?;

        let export = self.export_actions_jsonl(
            cwd,
            &ExportActionsRequest {
                capture_path: prepared.capture_path.clone(),
                output_dir: Some(prepared.output_dir.clone()),
                basename: Some(prepared.basename.clone()),
                only_drawcalls: req.only_drawcalls,
                marker_prefix: req.marker_prefix.clone(),
                event_id_min: req.event_id_min,
                event_id_max: req.event_id_max,
                name_contains: req.name_contains.clone(),
                marker_contains: req.marker_contains.clone(),
                case_sensitive: req.case_sensitive,
            },
        )?;

        Ok(CaptureAndExportActionsResponse {
            target_ident: prepared.target_ident,
            capture_path: export.capture_path,
            capture_file_template: prepared.capture_file_template,
            stdout: prepared.stdout,
            stderr: prepared.stderr,
            actions_jsonl_path: export.actions_jsonl_path,
            summary_json_path: export.summary_json_path,
            total_actions: export.total_actions,
            drawcall_actions: export.drawcall_actions,
        })
    }

    pub fn capture_and_export_bindings_index_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBindingsIndexRequest,
    ) -> Result<CaptureAndExportBindingsIndexResponse, CaptureAndExportBindingsIndexError> {
        let prepared = self.prepare_one_shot_capture(
            cwd,
            OneShotCaptureRequest {
                executable: &req.executable,
                args: &req.args,
                working_dir: req.working_dir.as_deref(),
                artifacts_dir: req.artifacts_dir.as_deref(),
                capture_template_name: req.capture_template_name.as_deref(),
                host: &req.host,
                num_frames: req.num_frames,
                timeout_s: req.timeout_s,
                output_dir: req.output_dir.as_deref(),
                basename: req.basename.as_deref(),
            },
        )?;

        let export = self.export_bindings_index_jsonl(
            cwd,
            &ExportBindingsIndexRequest {
                capture_path: prepared.capture_path.clone(),
                output_dir: Some(prepared.output_dir.clone()),
                basename: Some(prepared.basename.clone()),
                marker_prefix: req.marker_prefix.clone(),
                event_id_min: req.event_id_min,
                event_id_max: req.event_id_max,
                name_contains: req.name_contains.clone(),
                marker_contains: req.marker_contains.clone(),
                case_sensitive: req.case_sensitive,
                include_cbuffers: req.include_cbuffers,
                include_outputs: req.include_outputs,
            },
        )?;

        Ok(CaptureAndExportBindingsIndexResponse {
            target_ident: prepared.target_ident,
            capture_path: export.capture_path,
            capture_file_template: prepared.capture_file_template,
            stdout: prepared.stdout,
            stderr: prepared.stderr,
            bindings_jsonl_path: export.bindings_jsonl_path,
            summary_json_path: export.summary_json_path,
            total_drawcalls: export.total_drawcalls,
        })
    }

    pub fn capture_and_export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBundleRequest,
    ) -> Result<CaptureAndExportBundleResponse, CaptureAndExportBundleError> {
        let prepared = self.prepare_one_shot_capture(
            cwd,
            OneShotCaptureRequest {
                executable: &req.executable,
                args: &req.args,
                working_dir: req.working_dir.as_deref(),
                artifacts_dir: req.artifacts_dir.as_deref(),
                capture_template_name: req.capture_template_name.as_deref(),
                host: &req.host,
                num_frames: req.num_frames,
                timeout_s: req.timeout_s,
                output_dir: req.output_dir.as_deref(),
                basename: req.basename.as_deref(),
            },
        )?;

        let export = self.export_bundle_jsonl(
            cwd,
            &ExportBundleRequest {
                capture_path: prepared.capture_path.clone(),
                output_dir: Some(prepared.output_dir.clone()),
                basename: Some(prepared.basename.clone()),
                only_drawcalls: req.only_drawcalls,
                marker_prefix: req.marker_prefix.clone(),
                event_id_min: req.event_id_min,
                event_id_max: req.event_id_max,
                name_contains: req.name_contains.clone(),
                marker_contains: req.marker_contains.clone(),
                case_sensitive: req.case_sensitive,
                include_cbuffers: req.include_cbuffers,
                include_outputs: req.include_outputs,
            },
        )?;

        let mut thumbnail_output_path = None;
        if req.save_thumbnail {
            let output_path = req
                .thumbnail_output_path
                .as_deref()
                .map(|path| resolve_path_from_cwd(cwd, path))
                .unwrap_or_else(|| {
                    Path::new(&prepared.output_dir).join(format!("{}.thumb.png", prepared.basename))
                });

            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(CaptureAndExportBundleError::SaveThumbnail)?;
            }

            self.save_thumbnail(Path::new(&export.capture_path), &output_path)
                .map_err(CaptureAndExportBundleError::SaveThumbnail)?;
            thumbnail_output_path = Some(output_path.display().to_string());
        }

        let mut ui_pid = None;
        if req.open_capture_ui {
            let child = self.open_capture_in_ui(Path::new(&export.capture_path))?;
            ui_pid = Some(child.id());
        }

        Ok(CaptureAndExportBundleResponse {
            target_ident: prepared.target_ident,
            capture_path: export.capture_path,
            capture_file_template: prepared.capture_file_template,
            stdout: prepared.stdout,
            stderr: prepared.stderr,
            actions_jsonl_path: export.actions_jsonl_path,
            actions_summary_json_path: export.actions_summary_json_path,
            total_actions: export.total_actions,
            drawcall_actions: export.drawcall_actions,
            bindings_jsonl_path: export.bindings_jsonl_path,
            bindings_summary_json_path: export.bindings_summary_json_path,
            total_drawcalls: export.total_drawcalls,
            thumbnail_output_path,
            ui_pid,
        })
    }

    fn prepare_one_shot_capture(
        &self,
        cwd: &Path,
        req: OneShotCaptureRequest<'_>,
    ) -> Result<PreparedOneShotCapture, PrepareOneShotCaptureError> {
        let artifacts_dir = req
            .artifacts_dir
            .map(|path| resolve_path_from_cwd(cwd, path))
            .unwrap_or_else(|| default_artifacts_dir(cwd));
        std::fs::create_dir_all(&artifacts_dir)
            .map_err(PrepareOneShotCaptureError::CreateArtifactsDir)?;

        let capture_file_template = req
            .capture_template_name
            .map(|name| artifacts_dir.join(format!("{name}.rdc")));

        let launch = self.launch_capture(&CaptureLaunchRequest {
            executable: resolve_path_from_cwd(cwd, req.executable),
            args: req.args.iter().map(OsString::from).collect(),
            working_dir: req.working_dir.map(|path| resolve_path_from_cwd(cwd, path)),
            capture_file_template: capture_file_template.clone(),
        })?;

        let capture = self.trigger_capture_via_target_control(
            cwd,
            &TriggerCaptureRequest {
                host: req.host.to_string(),
                target_ident: launch.target_ident,
                num_frames: req.num_frames,
                timeout_s: req.timeout_s,
            },
        )?;

        let output_dir = req
            .output_dir
            .map(|path| resolve_path_from_cwd(cwd, path))
            .unwrap_or_else(|| default_exports_dir(cwd));
        std::fs::create_dir_all(&output_dir)
            .map_err(PrepareOneShotCaptureError::CreateOutputDir)?;

        let basename = req.basename.map(str::to_owned).unwrap_or_else(|| {
            Path::new(&capture.capture_path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("capture")
                .to_string()
        });

        Ok(PreparedOneShotCapture {
            target_ident: launch.target_ident,
            capture_path: capture.capture_path,
            capture_file_template: capture_file_template.map(|path| path.display().to_string()),
            stdout: launch.stdout,
            stderr: launch.stderr,
            output_dir: output_dir.display().to_string(),
            basename,
        })
    }
}
