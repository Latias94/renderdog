use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    BindingsExportOptions, CaptureInput, CapturePostActionOutputs, CapturePostActions,
    DrawcallScope, EventFilter, ExportActionsError, ExportActionsRequest, ExportBindingsIndexError,
    ExportBindingsIndexRequest, ExportBundleError, ExportBundleRequest, ExportOutput,
    LaunchCaptureError, OneShotCaptureOptions, OneShotCaptureTarget, RenderDocInstallation,
    TriggerCaptureError, TriggerCaptureRequest, default_exports_dir, resolve_path_from_cwd,
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
    #[serde(flatten)]
    pub post_actions: CapturePostActions,
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
    #[serde(flatten)]
    pub post_actions: CapturePostActionOutputs,
}

#[derive(Debug, Error)]
pub enum PrepareOneShotCaptureError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("launch capture failed: {0}")]
    Launch(#[from] LaunchCaptureError),
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
    target: &'a OneShotCaptureTarget,
    capture: &'a OneShotCaptureOptions,
    output: &'a ExportOutput,
}

impl<'a> OneShotCaptureRequest<'a> {
    fn new(
        target: &'a OneShotCaptureTarget,
        capture: &'a OneShotCaptureOptions,
        output: &'a ExportOutput,
    ) -> Self {
        Self {
            target,
            capture,
            output,
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
                        post_actions: req.post_actions.clone(),
                    },
                )?;

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
                    post_actions: export.post_actions,
                })
            },
        )
    }

    fn prepare_one_shot_capture(
        &self,
        cwd: &Path,
        req: OneShotCaptureRequest<'_>,
    ) -> Result<PreparedOneShotCapture, PrepareOneShotCaptureError> {
        let launch = self.launch_capture_in_cwd(cwd, req.target)?;

        let capture = self.trigger_capture_via_target_control(
            cwd,
            &TriggerCaptureRequest {
                host: req.capture.host.clone(),
                target_ident: launch.target_ident,
                num_frames: req.capture.num_frames,
                timeout_s: req.capture.timeout_s,
            },
        )?;

        let output_dir = req
            .output
            .output_dir
            .as_deref()
            .map(|path| resolve_path_from_cwd(cwd, path))
            .unwrap_or_else(|| default_exports_dir(cwd));
        std::fs::create_dir_all(&output_dir)
            .map_err(PrepareOneShotCaptureError::CreateOutputDir)?;

        let basename = req.output.basename.clone().unwrap_or_else(|| {
            Path::new(&capture.capture_path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("capture")
                .to_string()
        });

        Ok(PreparedOneShotCapture {
            target_ident: launch.target_ident,
            capture_path: PathBuf::from(capture.capture_path),
            capture_file_template: launch.capture_file_template.map(PathBuf::from),
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
