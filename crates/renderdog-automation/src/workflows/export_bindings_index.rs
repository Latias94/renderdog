use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobRequest, define_qrenderdoc_json_job_error};
use crate::{
    default_capture_basename, resolve_export_output_dir_from_cwd, resolve_path_string_from_cwd,
};

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
        let capture_path = resolve_path_string_from_cwd(cwd, &req.capture_path);
        let output_dir = resolve_export_output_dir_from_cwd(cwd, req.output_dir.as_deref());
        std::fs::create_dir_all(&output_dir).map_err(ExportBindingsIndexError::CreateOutputDir)?;
        let basename = req
            .basename
            .clone()
            .unwrap_or_else(|| default_capture_basename(&capture_path));
        let req = ExportBindingsIndexRequest {
            capture_path,
            output_dir: Some(output_dir.display().to_string()),
            basename: Some(basename),
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
