use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobError, QRenderDocJsonJobRequest};
use crate::{
    default_capture_basename, resolve_export_output_dir_from_cwd, resolve_path_string_from_cwd,
};

use super::{ExportActionsRequest, ExportActionsResponse};

#[derive(Debug, Error)]
pub enum ExportActionsError {
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

impl From<QRenderDocJsonJobError> for ExportActionsError {
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
    pub fn export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, ExportActionsError> {
        let capture_path = resolve_path_string_from_cwd(cwd, &req.capture_path);
        let output_dir = resolve_export_output_dir_from_cwd(cwd, req.output_dir.as_deref());
        std::fs::create_dir_all(&output_dir).map_err(ExportActionsError::CreateOutputDir)?;
        let basename = req
            .basename
            .clone()
            .unwrap_or_else(|| default_capture_basename(&capture_path));
        let req = ExportActionsRequest {
            capture_path,
            output_dir: Some(output_dir.display().to_string()),
            basename: Some(basename),
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
