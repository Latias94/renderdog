use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobRequest, define_qrenderdoc_json_job_error};
use crate::{
    CaptureInput, ExportOutput, default_capture_basename, resolve_export_output_dir_from_cwd,
    resolve_path_string_from_cwd,
};

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
        let capture_path = resolve_path_string_from_cwd(cwd, &req.capture.capture_path);
        let output_dir = resolve_export_output_dir_from_cwd(cwd, req.output.output_dir.as_deref());
        std::fs::create_dir_all(&output_dir).map_err(ExportActionsError::CreateOutputDir)?;
        let basename = req
            .output
            .basename
            .clone()
            .unwrap_or_else(|| default_capture_basename(&capture_path));
        let req = ExportActionsRequest {
            capture: CaptureInput { capture_path },
            output: ExportOutput {
                output_dir: Some(output_dir.display().to_string()),
                basename: Some(basename),
            },
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(
            cwd,
            &QRenderDocJsonJobRequest {
                run_dir_prefix: "export_actions_jsonl",
                script_file_name: "export_actions_jsonl.py",
                script_content: EXPORT_ACTIONS_JSONL_PY,
                request: &req,
            },
        )
        .map_err(ExportActionsError::from)
    }
}

const EXPORT_ACTIONS_JSONL_PY: &str = include_str!("../../scripts/export_actions_jsonl.py");
