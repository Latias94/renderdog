use std::path::Path;

use thiserror::Error;

use crate::scripting::{QRenderDocJsonEnvelope, create_qrenderdoc_run_dir};
use crate::{
    QRenderDocPythonRequest, RenderDocInstallation, default_scripts_dir, write_script_file,
};

use super::{TriggerCaptureRequest, TriggerCaptureResponse, util::remove_if_exists};

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

impl From<crate::QRenderDocPythonError> for TriggerCaptureError {
    fn from(value: crate::QRenderDocPythonError) -> Self {
        Self::QRenderDocPython(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn trigger_capture_via_target_control(
        &self,
        cwd: &Path,
        req: &TriggerCaptureRequest,
    ) -> Result<TriggerCaptureResponse, TriggerCaptureError> {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(TriggerCaptureError::CreateArtifactsDir)?;

        let script_path = scripts_dir.join("trigger_capture.py");
        write_script_file(&script_path, TRIGGER_CAPTURE_PY)
            .map_err(TriggerCaptureError::WriteScript)?;

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, "trigger_capture")
            .map_err(TriggerCaptureError::CreateArtifactsDir)?;
        let request_path = run_dir.join("trigger_capture.request.json");
        let response_path = run_dir.join("trigger_capture.response.json");
        remove_if_exists(&response_path).map_err(TriggerCaptureError::WriteRequest)?;
        std::fs::write(
            &request_path,
            serde_json::to_vec(req).map_err(TriggerCaptureError::ParseJson)?,
        )
        .map_err(TriggerCaptureError::WriteRequest)?;

        let result = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path: script_path.clone(),
            args: Vec::new(),
            working_dir: Some(run_dir.clone()),
        })?;
        let _ = result;
        let bytes = std::fs::read(&response_path).map_err(TriggerCaptureError::ReadResponse)?;
        let env: QRenderDocJsonEnvelope<TriggerCaptureResponse> =
            serde_json::from_slice(&bytes).map_err(TriggerCaptureError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| TriggerCaptureError::ScriptError("missing result".into()))
        } else {
            Err(TriggerCaptureError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }
}

const TRIGGER_CAPTURE_PY: &str = include_str!("../../scripts/trigger_capture.py");
