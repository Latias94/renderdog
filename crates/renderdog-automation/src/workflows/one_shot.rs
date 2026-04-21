use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    BundleExportArtifacts, BundleExportOptions, CaptureLaunchReport, CaptureRef,
    CaptureTargetError, CaptureTargetRequest, ExportBundleError, ExportBundleRequest,
    ExportBundleResponse, ExportOutput, RenderDocInstallation, TriggerCaptureError,
    TriggerCaptureOptions,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBundleRequest {
    #[serde(flatten)]
    pub target: CaptureTargetRequest,
    #[serde(flatten)]
    pub trigger: TriggerCaptureOptions,
    #[serde(flatten)]
    pub output: ExportOutput,
    #[serde(flatten)]
    pub bundle: BundleExportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CaptureAndExportBundleResponse {
    #[serde(flatten)]
    pub launch: CaptureLaunchReport,
    #[serde(flatten)]
    pub capture: CaptureRef,
    #[serde(flatten)]
    pub artifacts: BundleExportArtifacts,
}

#[derive(Debug, Error)]
pub enum OneShotCaptureError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("capture target failed: {0}")]
    CaptureTarget(#[from] CaptureTargetError),
    #[error("trigger capture failed: {0}")]
    Trigger(#[from] TriggerCaptureError),
}

#[derive(Debug, Error)]
pub enum CaptureAndExportBundleError {
    #[error(transparent)]
    Capture(#[from] OneShotCaptureError),
    #[error("export bundle failed: {0}")]
    Export(#[from] ExportBundleError),
}

struct CompletedOneShotCapture {
    launch: CaptureLaunchReport,
    export: ExportBundleRequest,
}

impl CompletedOneShotCapture {
    fn into_response(self, export: ExportBundleResponse) -> CaptureAndExportBundleResponse {
        let ExportBundleResponse { capture, artifacts } = export;

        CaptureAndExportBundleResponse {
            launch: self.launch,
            capture,
            artifacts,
        }
    }
}

impl RenderDocInstallation {
    pub fn capture_and_export_bundle_jsonl(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBundleRequest,
    ) -> Result<CaptureAndExportBundleResponse, CaptureAndExportBundleError> {
        let capture = self.capture_one_shot(cwd, req)?;
        let export = self.export_bundle_jsonl(cwd, &capture.export)?;

        Ok(capture.into_response(export))
    }

    fn capture_one_shot(
        &self,
        cwd: &Path,
        req: &CaptureAndExportBundleRequest,
    ) -> Result<CompletedOneShotCapture, OneShotCaptureError> {
        let resolved_target = req.target.resolved_in_cwd(cwd)?;
        let launched_target = self.launch_capture_target(&resolved_target)?;

        let triggered_capture = self.trigger_capture_via_target_control(
            cwd,
            &req.trigger.for_target(launched_target.target),
        )?;

        let (capture, output) = req
            .output
            .normalized_for_capture(cwd, &triggered_capture.capture)
            .map_err(OneShotCaptureError::CreateOutputDir)?;

        Ok(CompletedOneShotCapture {
            launch: launched_target,
            export: ExportBundleRequest {
                capture,
                output,
                bundle: req.bundle.clone(),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{
        CaptureAndExportBundleError, CaptureAndExportBundleRequest, CaptureAndExportBundleResponse,
        CompletedOneShotCapture, OneShotCaptureError,
    };
    use crate::{
        ActionsExportArtifacts, BindingsExportArtifacts, BindingsExportOptions,
        BundleExportArtifacts, BundleExportOptions, CaptureInput, CaptureLaunchReport,
        CapturePostActionOutputs, CapturePostActions, CaptureRef, CaptureTargetError,
        CaptureTargetRequest, DrawcallScope, EventFilter, ExportBundleRequest,
        ExportBundleResponse, ExportOutput, TargetControlRef, TriggerCaptureOptions,
    };

    #[test]
    fn completed_one_shot_capture_builds_export_bundle_request() {
        let capture = CompletedOneShotCapture {
            launch: CaptureLaunchReport {
                target: TargetControlRef::new(7),
                capture_file_template: Some("/tmp/frame".to_string()),
                stdout: "stdout".to_string(),
                stderr: "stderr".to_string(),
            },
            export: ExportBundleRequest {
                capture: CaptureInput {
                    capture_path: "/tmp/frame.rdc".to_string(),
                },
                output: ExportOutput {
                    output_dir: Some("/tmp/out".to_string()),
                    basename: Some("frame".to_string()),
                },
                bundle: BundleExportOptions {
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
                },
            },
        };

        assert_eq!(capture.export.capture.capture_path, "/tmp/frame.rdc");
        assert_eq!(
            capture.export.output.output_dir.as_deref(),
            Some("/tmp/out")
        );
        assert_eq!(capture.export.output.basename.as_deref(), Some("frame"));
        assert!(capture.export.bundle.drawcall_scope.only_drawcalls);
        assert_eq!(
            capture.export.bundle.filter.marker_contains.as_deref(),
            Some("fret")
        );
        assert!(capture.export.bundle.bindings.include_cbuffers);
        assert!(capture.export.bundle.post_actions.save_thumbnail);
    }

    #[test]
    fn completed_one_shot_capture_merges_export_response() {
        let capture = CompletedOneShotCapture {
            launch: CaptureLaunchReport {
                target: TargetControlRef::new(9),
                capture_file_template: Some("/tmp/frame".to_string()),
                stdout: "stdout".to_string(),
                stderr: "stderr".to_string(),
            },
            export: ExportBundleRequest {
                capture: CaptureInput {
                    capture_path: "/tmp/frame.rdc".to_string(),
                },
                output: ExportOutput {
                    output_dir: Some("/tmp/out".to_string()),
                    basename: Some("frame".to_string()),
                },
                bundle: BundleExportOptions {
                    drawcall_scope: DrawcallScope {
                        only_drawcalls: false,
                    },
                    filter: EventFilter::default(),
                    bindings: BindingsExportOptions {
                        include_cbuffers: false,
                        include_outputs: false,
                    },
                    post_actions: CapturePostActions::default(),
                },
            },
        };
        let export = ExportBundleResponse {
            capture: CaptureRef::new("/tmp/frame.rdc"),
            artifacts: BundleExportArtifacts {
                actions: ActionsExportArtifacts {
                    actions_jsonl_path: "/tmp/out/frame.actions.jsonl".to_string(),
                    actions_summary_json_path: "/tmp/out/frame.summary.json".to_string(),
                    total_actions: 10,
                    drawcall_actions: 4,
                },
                bindings: BindingsExportArtifacts {
                    bindings_jsonl_path: "/tmp/out/frame.bindings.jsonl".to_string(),
                    bindings_summary_json_path: "/tmp/out/frame.bindings_summary.json".to_string(),
                    total_drawcalls: 4,
                },
                post_actions: CapturePostActionOutputs {
                    thumbnail_output_path: Some("/tmp/out/frame.thumb.png".to_string()),
                    ui_pid: Some(123),
                },
            },
        };

        let response: CaptureAndExportBundleResponse = capture.into_response(export);

        assert_eq!(response.launch.target.target_ident, 9);
        assert_eq!(response.capture.capture_path, "/tmp/frame.rdc");
        assert_eq!(
            response.launch.capture_file_template.as_deref(),
            Some("/tmp/frame")
        );
        assert_eq!(response.launch.stdout, "stdout");
        assert_eq!(response.launch.stderr, "stderr");
        assert_eq!(
            response.artifacts.actions.actions_jsonl_path,
            "/tmp/out/frame.actions.jsonl"
        );
        assert_eq!(response.artifacts.bindings.total_drawcalls, 4);
        assert_eq!(
            response
                .artifacts
                .post_actions
                .thumbnail_output_path
                .as_deref(),
            Some("/tmp/out/frame.thumb.png")
        );
        assert_eq!(response.artifacts.post_actions.ui_pid, Some(123));
    }

    #[test]
    fn capture_and_export_bundle_request_serializes_bundle_options_flattened() {
        let req = CaptureAndExportBundleRequest {
            target: CaptureTargetRequest {
                executable: "game".to_string(),
                args: vec!["--flag".to_string()],
                working_dir: None,
                artifacts_dir: None,
                capture_template_name: Some("capture".to_string()),
            },
            trigger: TriggerCaptureOptions::default(),
            output: ExportOutput::default(),
            bundle: BundleExportOptions {
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
            },
        };

        let json = serde_json::to_value(req).expect("serialize request");
        let object = json.as_object().expect("request object");

        assert_eq!(
            object.get("executable"),
            Some(&Value::String("game".to_string()))
        );
        assert_eq!(
            object.get("capture_template_name"),
            Some(&Value::String("capture".to_string()))
        );
        assert_eq!(
            object.get("host"),
            Some(&Value::String("localhost".to_string()))
        );
        assert_eq!(object.get("num_frames"), Some(&Value::Number(1_u32.into())));
        assert_eq!(object.get("timeout_s"), Some(&Value::Number(60_u32.into())));
        assert_eq!(object.get("only_drawcalls"), Some(&Value::Bool(true)));
        assert_eq!(
            object.get("marker_contains"),
            Some(&Value::String("fret".to_string()))
        );
        assert_eq!(object.get("include_cbuffers"), Some(&Value::Bool(true)));
        assert_eq!(object.get("save_thumbnail"), Some(&Value::Bool(true)));
        assert!(!object.contains_key("bundle"));
    }

    #[test]
    fn capture_and_export_bundle_response_serializes_artifacts_flattened() {
        let response = CaptureAndExportBundleResponse {
            launch: CaptureLaunchReport {
                target: TargetControlRef::new(9),
                capture_file_template: Some("/tmp/frame".to_string()),
                stdout: "stdout".to_string(),
                stderr: "stderr".to_string(),
            },
            capture: CaptureRef::new("/tmp/frame.rdc"),
            artifacts: BundleExportArtifacts {
                actions: ActionsExportArtifacts {
                    actions_jsonl_path: "/tmp/out/frame.actions.jsonl".to_string(),
                    actions_summary_json_path: "/tmp/out/frame.summary.json".to_string(),
                    total_actions: 10,
                    drawcall_actions: 4,
                },
                bindings: BindingsExportArtifacts {
                    bindings_jsonl_path: "/tmp/out/frame.bindings.jsonl".to_string(),
                    bindings_summary_json_path: "/tmp/out/frame.bindings_summary.json".to_string(),
                    total_drawcalls: 4,
                },
                post_actions: CapturePostActionOutputs {
                    thumbnail_output_path: Some("/tmp/out/frame.thumb.png".to_string()),
                    ui_pid: Some(123),
                },
            },
        };

        let json = serde_json::to_value(response).expect("serialize response");
        let object = json.as_object().expect("response object");

        assert_eq!(
            object.get("target_ident"),
            Some(&Value::Number(9_u32.into()))
        );
        assert_eq!(
            object.get("capture_path"),
            Some(&Value::String("/tmp/frame.rdc".to_string()))
        );
        assert_eq!(
            object.get("capture_file_template"),
            Some(&Value::String("/tmp/frame".to_string()))
        );
        assert_eq!(
            object.get("stdout"),
            Some(&Value::String("stdout".to_string()))
        );
        assert_eq!(
            object.get("stderr"),
            Some(&Value::String("stderr".to_string()))
        );
        assert_eq!(
            object.get("actions_jsonl_path"),
            Some(&Value::String("/tmp/out/frame.actions.jsonl".to_string()))
        );
        assert_eq!(
            object.get("bindings_jsonl_path"),
            Some(&Value::String("/tmp/out/frame.bindings.jsonl".to_string()))
        );
        assert_eq!(
            object.get("thumbnail_output_path"),
            Some(&Value::String("/tmp/out/frame.thumb.png".to_string()))
        );
        assert!(!object.contains_key("launch"));
        assert!(!object.contains_key("capture"));
        assert!(!object.contains_key("artifacts"));
        assert!(!object.contains_key("actions"));
        assert!(!object.contains_key("bindings"));
    }

    #[test]
    fn one_shot_capture_error_wraps_capture_target_error() {
        let err: OneShotCaptureError = CaptureTargetError::InvalidTargetIdent(-1).into();

        assert!(matches!(
            err,
            OneShotCaptureError::CaptureTarget(CaptureTargetError::InvalidTargetIdent(-1))
        ));
    }

    #[test]
    fn capture_and_export_bundle_error_wraps_one_shot_capture_error() {
        let err: CaptureAndExportBundleError = OneShotCaptureError::CreateOutputDir(
            std::io::Error::from(std::io::ErrorKind::PermissionDenied),
        )
        .into();

        assert!(matches!(
            err,
            CaptureAndExportBundleError::Capture(OneShotCaptureError::CreateOutputDir(_))
        ));
    }
}
