use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    BindingsExportOptions, CaptureInput, CaptureLaunchError, CaptureLaunchRequest, DrawcallScope,
    EventFilter, ExportActionsError, ExportActionsRequest, ExportBindingsIndexError,
    ExportBindingsIndexRequest, ExportBundleError, ExportBundleRequest, ExportOutput,
    OneShotCaptureOptions, OneShotCaptureTarget, OpenCaptureUiError, RenderDocInstallation,
    TriggerCaptureError, TriggerCaptureRequest, default_artifacts_dir, default_exports_dir,
    resolve_path_from_cwd,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportActionsRequest {
    #[serde(flatten)]
    pub target: OneShotCaptureTarget,
    #[serde(flatten)]
    pub capture: OneShotCaptureOptions,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBindingsIndexRequest {
    #[serde(flatten)]
    pub target: OneShotCaptureTarget,
    #[serde(flatten)]
    pub capture: OneShotCaptureOptions,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBundleRequest {
    #[serde(flatten)]
    pub target: OneShotCaptureTarget,
    #[serde(flatten)]
    pub capture: OneShotCaptureOptions,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub drawcall_scope: DrawcallScope,
    #[serde(flatten)]
    pub filter: EventFilter,
    #[serde(flatten)]
    pub bindings: BindingsExportOptions,
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

struct OneShotCaptureResponseBase {
    target_ident: u32,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,
}

struct PreparedOneShotCapture {
    target_ident: u32,
    capture_path: PathBuf,
    capture_file_template: Option<PathBuf>,
    stdout: String,
    stderr: String,
    output_dir: PathBuf,
    basename: String,
}

impl PreparedOneShotCapture {
    fn capture_input(&self) -> CaptureInput {
        CaptureInput {
            capture_path: self.capture_path.display().to_string(),
        }
    }

    fn export_output(&self) -> ExportOutput {
        ExportOutput {
            output_dir: Some(self.output_dir.display().to_string()),
            basename: Some(self.basename.clone()),
        }
    }

    fn default_thumbnail_output_path(&self) -> PathBuf {
        self.output_dir.join(format!("{}.thumb.png", self.basename))
    }

    fn into_response_base(self) -> OneShotCaptureResponseBase {
        OneShotCaptureResponseBase {
            target_ident: self.target_ident,
            capture_file_template: self
                .capture_file_template
                .map(|path| path.display().to_string()),
            stdout: self.stdout,
            stderr: self.stderr,
        }
    }
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

impl<'a> OneShotCaptureRequest<'a> {
    fn new(
        target: &'a OneShotCaptureTarget,
        capture: &'a OneShotCaptureOptions,
        output: &'a ExportOutput,
    ) -> Self {
        Self {
            executable: &target.executable,
            args: &target.args,
            working_dir: target.working_dir.as_deref(),
            artifacts_dir: target.artifacts_dir.as_deref(),
            capture_template_name: target.capture_template_name.as_deref(),
            host: &capture.host,
            num_frames: capture.num_frames,
            timeout_s: capture.timeout_s,
            output_dir: output.output_dir.as_deref(),
            basename: output.basename.as_deref(),
        }
    }
}

impl RenderDocInstallation {
    pub fn capture_and_export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportActionsRequest,
    ) -> Result<CaptureAndExportActionsResponse, CaptureAndExportActionsError> {
        self.with_prepared_one_shot_capture(
            cwd,
            OneShotCaptureRequest::new(&req.target, &req.capture, &req.output),
            |install, prepared| {
                let export = install.export_actions_jsonl(
                    cwd,
                    &ExportActionsRequest {
                        capture: prepared.capture_input(),
                        output: prepared.export_output(),
                        drawcall_scope: req.drawcall_scope,
                        filter: req.filter.clone(),
                    },
                )?;

                let base = prepared.into_response_base();
                Ok(CaptureAndExportActionsResponse {
                    target_ident: base.target_ident,
                    capture_path: export.capture_path,
                    capture_file_template: base.capture_file_template,
                    stdout: base.stdout,
                    stderr: base.stderr,
                    actions_jsonl_path: export.actions_jsonl_path,
                    summary_json_path: export.summary_json_path,
                    total_actions: export.total_actions,
                    drawcall_actions: export.drawcall_actions,
                })
            },
        )
    }

    pub fn capture_and_export_bindings_index_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBindingsIndexRequest,
    ) -> Result<CaptureAndExportBindingsIndexResponse, CaptureAndExportBindingsIndexError> {
        self.with_prepared_one_shot_capture(
            cwd,
            OneShotCaptureRequest::new(&req.target, &req.capture, &req.output),
            |install, prepared| {
                let export = install.export_bindings_index_jsonl(
                    cwd,
                    &ExportBindingsIndexRequest {
                        capture: prepared.capture_input(),
                        output: prepared.export_output(),
                        filter: req.filter.clone(),
                        bindings: req.bindings,
                    },
                )?;

                let base = prepared.into_response_base();
                Ok(CaptureAndExportBindingsIndexResponse {
                    target_ident: base.target_ident,
                    capture_path: export.capture_path,
                    capture_file_template: base.capture_file_template,
                    stdout: base.stdout,
                    stderr: base.stderr,
                    bindings_jsonl_path: export.bindings_jsonl_path,
                    summary_json_path: export.summary_json_path,
                    total_drawcalls: export.total_drawcalls,
                })
            },
        )
    }

    pub fn capture_and_export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBundleRequest,
    ) -> Result<CaptureAndExportBundleResponse, CaptureAndExportBundleError> {
        self.with_prepared_one_shot_capture(
            cwd,
            OneShotCaptureRequest::new(&req.target, &req.capture, &req.output),
            |install, prepared| {
                let export = install.export_bundle_jsonl(
                    cwd,
                    &ExportBundleRequest {
                        capture: prepared.capture_input(),
                        output: prepared.export_output(),
                        drawcall_scope: req.drawcall_scope,
                        filter: req.filter.clone(),
                        bindings: req.bindings,
                    },
                )?;

                let mut thumbnail_output_path = None;
                if req.save_thumbnail {
                    let output_path = req
                        .thumbnail_output_path
                        .as_deref()
                        .map(|path| resolve_path_from_cwd(cwd, path))
                        .unwrap_or_else(|| prepared.default_thumbnail_output_path());

                    if let Some(parent) = output_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(CaptureAndExportBundleError::SaveThumbnail)?;
                    }

                    install
                        .save_thumbnail(prepared.capture_path.as_path(), &output_path)
                        .map_err(CaptureAndExportBundleError::SaveThumbnail)?;
                    thumbnail_output_path = Some(output_path.display().to_string());
                }

                let mut ui_pid = None;
                if req.open_capture_ui {
                    let child = install.open_capture_in_ui(prepared.capture_path.as_path())?;
                    ui_pid = Some(child.id());
                }

                let base = prepared.into_response_base();
                Ok(CaptureAndExportBundleResponse {
                    target_ident: base.target_ident,
                    capture_path: export.capture_path,
                    capture_file_template: base.capture_file_template,
                    stdout: base.stdout,
                    stderr: base.stderr,
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
            },
        )
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
            capture_path: PathBuf::from(capture.capture_path),
            capture_file_template,
            stdout: launch.stdout,
            stderr: launch.stderr,
            output_dir,
            basename,
        })
    }

    fn with_prepared_one_shot_capture<T, E, F>(
        &self,
        cwd: &Path,
        req: OneShotCaptureRequest<'_>,
        op: F,
    ) -> Result<T, E>
    where
        E: From<PrepareOneShotCaptureError>,
        F: FnOnce(&RenderDocInstallation, PreparedOneShotCapture) -> Result<T, E>,
    {
        let prepared = self.prepare_one_shot_capture(cwd, req)?;
        op(self, prepared)
    }
}
