use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobError, QRenderDocJsonJobRequest};

use super::{TriggerCaptureRequest, TriggerCaptureResponse};

#[derive(Debug, Error)]
pub enum TriggerCaptureError {
    #[error("failed to create artifacts dir: {0}")]
    CreateArtifactsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc python failed: {0}")]
    QRenderDocPython(Box<crate::QRenderDocPythonError>),
    #[error("failed to parse capture JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<QRenderDocJsonJobError> for TriggerCaptureError {
    fn from(value: QRenderDocJsonJobError) -> Self {
        match value {
            QRenderDocJsonJobError::CreateScriptsDir(err) => Self::CreateArtifactsDir(err),
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
    pub fn trigger_capture_via_target_control(
        &self,
        cwd: &Path,
        req: &TriggerCaptureRequest,
    ) -> Result<TriggerCaptureResponse, TriggerCaptureError> {
        self.run_qrenderdoc_json_job(
            cwd,
            &QRenderDocJsonJobRequest {
                run_dir_prefix: "trigger_capture",
                script_file_name: "trigger_capture.py",
                script_content: TRIGGER_CAPTURE_PY,
                request: req,
            },
        )
        .map_err(TriggerCaptureError::from)
    }
}

const TRIGGER_CAPTURE_PY: &str = include_str!("../../scripts/trigger_capture.py");
