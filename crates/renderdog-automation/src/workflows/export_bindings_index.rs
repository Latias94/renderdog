use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::resolve_path_string_from_cwd;
use crate::scripting::{QRenderDocJsonJobError, QRenderDocJsonJobRequest};

use super::{ExportBindingsIndexRequest, ExportBindingsIndexResponse};

#[derive(Debug, Error)]
pub enum ExportBindingsIndexError {
    #[error("failed to create output dir: {0}")]
    CreateOutputDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse export JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<QRenderDocJsonJobError> for ExportBindingsIndexError {
    fn from(value: QRenderDocJsonJobError) -> Self {
        match value {
            QRenderDocJsonJobError::CreateScriptsDir(err) => Self::CreateOutputDir(err),
            QRenderDocJsonJobError::WriteScript(err) => Self::WriteScript(err),
            QRenderDocJsonJobError::WriteRequest(err) => Self::WriteRequest(err),
            QRenderDocJsonJobError::QRenderDocPython(err) => Self::QRenderDocPython(err),
            QRenderDocJsonJobError::ReadResponse(err) => Self::ReadResponse(err),
            QRenderDocJsonJobError::ParseJson(err) => Self::ParseJson(err),
            QRenderDocJsonJobError::ScriptError(err) => Self::ScriptError(err),
        }
    }
}

impl RenderDocInstallation {
    pub fn export_bindings_index_jsonl(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, ExportBindingsIndexError> {
        let req = ExportBindingsIndexRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_dir: resolve_path_string_from_cwd(cwd, &req.output_dir),
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
