use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    BindingsExportOptions, CaptureInput, CapturePostActionOutputs, CapturePostActions,
    CaptureTargetRequest, DrawcallScope, EventFilter, ExportBundleError, ExportBundleRequest,
    ExportBundleResponse, ExportOutput, OneShotCaptureTarget, OneShotTriggerOptions,
    RenderDocInstallation, ToolInvocationError, TriggerCaptureError, TriggerCaptureRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBundleRequest {
    #[serde(flatten)]
    pub target: OneShotCaptureTarget,
    #[serde(flatten)]
    pub trigger: OneShotTriggerOptions,
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
    #[error("failed to create capture artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("failed to launch capture target: {0}")]
    LaunchTarget(Box<ToolInvocationError>),
    #[error("renderdoccmd returned invalid target ident: {0}")]
    InvalidTargetIdent(i32),
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
    fn export_bundle_request(&self, req: &CaptureAndExportBundleRequest) -> ExportBundleRequest {
        ExportBundleRequest {
            capture: self.capture.clone(),
            output: self.output.clone(),
            drawcall_scope: req.drawcall_scope,
            filter: req.filter.clone(),
            bindings: req.bindings,
            post_actions: req.post_actions.clone(),
        }
    }

    fn into_response(self, export: ExportBundleResponse) -> CaptureAndExportBundleResponse {
        CaptureAndExportBundleResponse {
            target_ident: self.target_ident,
            capture_path: export.capture_path,
            capture_file_template: self.capture_file_template,
            stdout: self.stdout,
            stderr: self.stderr,
            actions_jsonl_path: export.actions_jsonl_path,
            actions_summary_json_path: export.actions_summary_json_path,
            total_actions: export.total_actions,
            drawcall_actions: export.drawcall_actions,
            bindings_jsonl_path: export.bindings_jsonl_path,
            bindings_summary_json_path: export.bindings_summary_json_path,
            total_drawcalls: export.total_drawcalls,
            post_actions: export.post_actions,
        }
    }
}

impl From<&OneShotCaptureTarget> for CaptureTargetRequest {
    fn from(value: &OneShotCaptureTarget) -> Self {
        Self {
            executable: value.executable.clone(),
            args: value.args.clone(),
            working_dir: value.working_dir.clone(),
            artifacts_dir: value.artifacts_dir.clone(),
            capture_template_name: value.capture_template_name.clone(),
        }
    }
}

fn map_capture_target_error(err: crate::capture::CaptureTargetError) -> PrepareOneShotCaptureError {
    match err {
        crate::capture::CaptureTargetError::CreateArtifactsDir(source) => {
            PrepareOneShotCaptureError::CreateArtifactsDir(source)
        }
        crate::capture::CaptureTargetError::Tool(source) => {
            PrepareOneShotCaptureError::LaunchTarget(source)
        }
        crate::capture::CaptureTargetError::InvalidTargetIdent(code) => {
            PrepareOneShotCaptureError::InvalidTargetIdent(code)
        }
    }
}

impl RenderDocInstallation {
    pub fn capture_and_export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBundleRequest,
    ) -> Result<CaptureAndExportBundleResponse, CaptureAndExportBundleError> {
        let prepared =
            self.prepare_one_shot_capture(cwd, &req.target, &req.trigger, &req.output)?;
        let export = self.export_bundle_jsonl(cwd, &prepared.export_bundle_request(req))?;

        Ok(prepared.into_response(export))
    }

    fn prepare_one_shot_capture(
        &self,
        cwd: &Path,
        target: &OneShotCaptureTarget,
        trigger_options: &OneShotTriggerOptions,
        output: &ExportOutput,
    ) -> Result<PreparedOneShotCapture, PrepareOneShotCaptureError> {
        let target_request = CaptureTargetRequest::from(target);
        let prepared_target = self
            .prepare_capture_target(cwd, &target_request)
            .map_err(map_capture_target_error)?;
        let launched_target = self
            .launch_prepared_capture_target(&prepared_target)
            .map_err(map_capture_target_error)?;

        let capture = self.trigger_capture_via_target_control(
            cwd,
            &TriggerCaptureRequest {
                host: trigger_options.host.clone(),
                target_ident: launched_target.target_ident,
                num_frames: trigger_options.num_frames,
                timeout_s: trigger_options.timeout_s,
            },
        )?;

        let (capture, output) = output
            .normalized_for_capture(
                cwd,
                &CaptureInput {
                    capture_path: capture.capture_path,
                },
            )
            .map_err(PrepareOneShotCaptureError::CreateOutputDir)?;

        Ok(PreparedOneShotCapture {
            target_ident: launched_target.target_ident,
            capture,
            output,
            capture_file_template: launched_target.capture_file_template,
            stdout: launched_target.stdout,
            stderr: launched_target.stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CaptureAndExportBundleRequest, CaptureAndExportBundleResponse, OneShotCaptureTarget,
        OneShotTriggerOptions, PreparedOneShotCapture,
    };
    use crate::{
        BindingsExportOptions, CaptureInput, CapturePostActionOutputs, CapturePostActions,
        DrawcallScope, EventFilter, ExportBundleResponse, ExportOutput,
    };

    #[test]
    fn prepared_one_shot_capture_builds_export_bundle_request() {
        let prepared = PreparedOneShotCapture {
            target_ident: 7,
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            output: ExportOutput {
                output_dir: Some("/tmp/out".to_string()),
                basename: Some("frame".to_string()),
            },
            capture_file_template: Some("/tmp/frame".to_string()),
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
        };
        let req = CaptureAndExportBundleRequest {
            target: OneShotCaptureTarget {
                executable: "game".to_string(),
                args: vec!["--flag".to_string()],
                working_dir: None,
                artifacts_dir: None,
                capture_template_name: None,
            },
            trigger: OneShotTriggerOptions::default(),
            output: ExportOutput::default(),
            drawcall_scope: DrawcallScope {
                only_drawcalls: true,
            },
            filter: EventFilter {
                marker_contains: Some("fret".to_string()),
                ..EventFilter::default()
            },
            bindings: BindingsExportOptions {
                include_cbuffers: true,
                include_outputs: false,
            },
            post_actions: CapturePostActions {
                save_thumbnail: true,
                thumbnail_output_path: Some("thumb.png".to_string()),
                open_capture_ui: false,
            },
        };

        let export_req = prepared.export_bundle_request(&req);

        assert_eq!(export_req.capture.capture_path, "/tmp/frame.rdc");
        assert_eq!(export_req.output.output_dir.as_deref(), Some("/tmp/out"));
        assert_eq!(export_req.output.basename.as_deref(), Some("frame"));
        assert!(export_req.drawcall_scope.only_drawcalls);
        assert_eq!(export_req.filter.marker_contains.as_deref(), Some("fret"));
        assert!(export_req.bindings.include_cbuffers);
        assert!(export_req.post_actions.save_thumbnail);
    }

    #[test]
    fn prepared_one_shot_capture_merges_export_response() {
        let prepared = PreparedOneShotCapture {
            target_ident: 9,
            capture: CaptureInput {
                capture_path: "/tmp/frame.rdc".to_string(),
            },
            output: ExportOutput {
                output_dir: Some("/tmp/out".to_string()),
                basename: Some("frame".to_string()),
            },
            capture_file_template: Some("/tmp/frame".to_string()),
            stdout: "stdout".to_string(),
            stderr: "stderr".to_string(),
        };
        let export = ExportBundleResponse {
            capture_path: "/tmp/frame.rdc".to_string(),
            actions_jsonl_path: "/tmp/out/frame.actions.jsonl".to_string(),
            actions_summary_json_path: "/tmp/out/frame.summary.json".to_string(),
            total_actions: 10,
            drawcall_actions: 4,
            bindings_jsonl_path: "/tmp/out/frame.bindings.jsonl".to_string(),
            bindings_summary_json_path: "/tmp/out/frame.bindings_summary.json".to_string(),
            total_drawcalls: 4,
            post_actions: CapturePostActionOutputs {
                thumbnail_output_path: Some("/tmp/out/frame.thumb.png".to_string()),
                ui_pid: Some(123),
            },
        };

        let response: CaptureAndExportBundleResponse = prepared.into_response(export);

        assert_eq!(response.target_ident, 9);
        assert_eq!(response.capture_path, "/tmp/frame.rdc");
        assert_eq!(
            response.capture_file_template.as_deref(),
            Some("/tmp/frame")
        );
        assert_eq!(response.actions_jsonl_path, "/tmp/out/frame.actions.jsonl");
        assert_eq!(response.total_drawcalls, 4);
        assert_eq!(
            response.post_actions.thumbnail_output_path.as_deref(),
            Some("/tmp/out/frame.thumb.png")
        );
        assert_eq!(response.post_actions.ui_pid, Some(123));
    }
}
