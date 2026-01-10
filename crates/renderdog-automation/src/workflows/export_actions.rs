use std::path::Path;

use thiserror::Error;

use crate::resolve_path_string_from_cwd;
use crate::scripting::{QRenderDocJsonEnvelope, create_qrenderdoc_run_dir};
use crate::{
    QRenderDocPythonRequest, RenderDocInstallation, default_scripts_dir, write_script_file,
};

use super::{ExportActionsRequest, ExportActionsResponse, util::remove_if_exists};

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

impl From<crate::QRenderDocPythonError> for ExportActionsError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, ExportActionsError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(ExportActionsError::CreateOutputDir)?;

        let script_path = scripts_dir.join("export_actions_jsonl.py");
        write_script_file(&script_path, EXPORT_ACTIONS_JSONL_PY)
            .map_err(ExportActionsError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "export_actions_jsonl")
            .map_err(ExportActionsError::CreateOutputDir)?;
        let request_path = run_dir.join("export_actions_jsonl.request.json");
        let response_path = run_dir.join("export_actions_jsonl.response.json");
        remove_if_exists(&response_path).map_err(ExportActionsError::WriteRequest)?;

        let req = ExportActionsRequest {
            capture_path: resolve_path_string_from_cwd(cwd, &req.capture_path),
            output_dir: resolve_path_string_from_cwd(cwd, &req.output_dir),
            ..req.clone()
        };

        std::fs::write(
            &request_path,
            serde_json::to_vec(&req).map_err(ExportActionsError::ParseJson)?,
        )
        .map_err(ExportActionsError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;
        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(ExportActionsError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<ExportActionsResponse> =
            serde_json::from_slice(&bytes).map_err(ExportActionsError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| ExportActionsError::ScriptError("missing result".into()))
        } else {
            Err(ExportActionsError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }
}

const EXPORT_ACTIONS_JSONL_PY: &str = include_str!("../../scripts/export_actions_jsonl.py");
