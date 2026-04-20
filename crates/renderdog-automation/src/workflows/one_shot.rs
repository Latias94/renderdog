use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    BindingsExportOptions, CaptureInput, CapturePostActionOutputs, CapturePostActions,
    DrawcallScope, EventFilter, ExportBundleError, ExportBundleRequest, ExportOutput,
    LaunchCaptureError, OneShotCaptureOptions, OneShotCaptureTarget, RenderDocInstallation,
    TriggerCaptureError, TriggerCaptureRequest, prepare_export_target,
};

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
pub enum CaptureAndExportBundleError {
    #[error(transparent)]
    Prepare(#[from] PrepareOneShotCaptureError),
    #[error("export bundle failed: {0}")]
    Export(#[from] ExportBundleError),
}

struct PreparedOneShotCapture {
    target_ident: u32,
    capture: CaptureInput,
    output: ExportOutput,
    capture_file_template: Option<String>,
    stdout: String,
    stderr: String,
}

impl PreparedOneShotCapture {
    fn capture_input(&self) -> CaptureInput {
        self.capture.clone()
    }

    fn export_output(&self) -> ExportOutput {
        self.output.clone()
    }
}

impl RenderDocInstallation {
    pub fn capture_and_export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBundleRequest,
    ) -> Result<CaptureAndExportBundleResponse, CaptureAndExportBundleError> {
        let prepared =
            self.prepare_one_shot_capture(cwd, &req.target, &req.capture, &req.output)?;
        let export = self.export_bundle_jsonl(
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
            post_actions: export.post_actions,
        })
    }

    fn prepare_one_shot_capture(
        &self,
        cwd: &Path,
        target: &OneShotCaptureTarget,
        capture_options: &OneShotCaptureOptions,
        output: &ExportOutput,
    ) -> Result<PreparedOneShotCapture, PrepareOneShotCaptureError> {
        let launch = self.prepare_launch_capture_request(cwd, target)?;
        let launch = self.launch_capture_prepared(&launch)?;

        let capture = self.trigger_capture_via_target_control(
            cwd,
            &TriggerCaptureRequest {
                host: capture_options.host.clone(),
                target_ident: launch.target_ident,
                num_frames: capture_options.num_frames,
                timeout_s: capture_options.timeout_s,
            },
        )?;

        let prepared_export = prepare_export_target(
            cwd,
            &capture.capture_path,
            output.output_dir.as_deref(),
            output.basename.as_deref(),
        )
        .map_err(PrepareOneShotCaptureError::CreateOutputDir)?;

        Ok(PreparedOneShotCapture {
            target_ident: launch.target_ident,
            capture: CaptureInput {
                capture_path: prepared_export.capture_path,
            },
            output: ExportOutput {
                output_dir: Some(prepared_export.output_dir),
                basename: Some(prepared_export.basename),
            },
            capture_file_template: launch.capture_file_template,
            stdout: launch.stdout,
            stderr: launch.stderr,
        })
    }
}
