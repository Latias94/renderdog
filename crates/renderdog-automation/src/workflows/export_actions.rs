use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJob, define_qrenderdoc_json_job_error};
use crate::{CaptureInput, ExportOutput, prepare_export_target};

use super::{ExportActionsRequest, ExportActionsResponse};

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum ExportActionsError {
        create_dir_variant: CreateOutputDir => "failed to create output dir: {0}",
        parse_json_message: "failed to parse export JSON: {0}",
    }
}

impl RenderDocInstallation {
    pub fn export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, ExportActionsError> {
        let prepared = prepare_export_target(
            cwd,
            &req.capture.capture_path,
            req.output.output_dir.as_deref(),
            req.output.basename.as_deref(),
        )
        .map_err(ExportActionsError::CreateOutputDir)?;

        let req = ExportActionsRequest {
            capture: CaptureInput {
                capture_path: prepared.capture_path,
            },
            output: ExportOutput {
                output_dir: Some(prepared.output_dir),
                basename: Some(prepared.basename),
            },
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(cwd, EXPORT_ACTIONS_JSONL_JOB, &req)
            .map_err(ExportActionsError::from)
    }
}

const EXPORT_ACTIONS_JSONL_PY: &str = include_str!("../../scripts/export_actions_jsonl.py");

const EXPORT_ACTIONS_JSONL_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "export_actions_jsonl",
    "export_actions_jsonl.py",
    EXPORT_ACTIONS_JSONL_PY,
);
