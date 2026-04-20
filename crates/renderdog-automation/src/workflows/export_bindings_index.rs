use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobRequest, define_qrenderdoc_json_job_error};
use crate::{CaptureInput, ExportOutput, prepare_export_target};

use super::{ExportBindingsIndexRequest, ExportBindingsIndexResponse};

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum ExportBindingsIndexError {
        create_dir_variant: CreateOutputDir => "failed to create output dir: {0}",
        parse_json_message: "failed to parse export JSON: {0}",
    }
}

impl RenderDocInstallation {
    pub fn export_bindings_index_jsonl(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, ExportBindingsIndexError> {
        let prepared = prepare_export_target(
            cwd,
            &req.capture.capture_path,
            req.output.output_dir.as_deref(),
            req.output.basename.as_deref(),
        )
        .map_err(ExportBindingsIndexError::CreateOutputDir)?;

        let req = ExportBindingsIndexRequest {
            capture: CaptureInput {
                capture_path: prepared.capture_path,
            },
            output: ExportOutput {
                output_dir: Some(prepared.output_dir),
                basename: Some(prepared.basename),
            },
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(
            cwd,
            &QRenderDocJsonJobRequest {
                run_dir_prefix: "export_bindings_index_jsonl",
                script_file_name: "export_bindings_index_jsonl.py",
                script_content: EXPORT_BINDINGS_INDEX_JSONL_PY,
                request: &req,
            },
        )
        .map_err(ExportBindingsIndexError::from)
    }
}

const EXPORT_BINDINGS_INDEX_JSONL_PY: &str =
    include_str!("../../scripts/export_bindings_index_jsonl.py");
