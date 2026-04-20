use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJob, define_qrenderdoc_json_job_error};

use super::{TriggerCaptureRequest, TriggerCaptureResponse};

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum TriggerCaptureError {
        create_dir_variant: CreateArtifactsDir => "failed to create artifacts dir: {0}",
        parse_json_message: "failed to parse capture JSON: {0}",
    }
}

impl RenderDocInstallation {
    pub fn trigger_capture_via_target_control(
        &self,
        cwd: &Path,
        req: &TriggerCaptureRequest,
    ) -> Result<TriggerCaptureResponse, TriggerCaptureError> {
        self.run_qrenderdoc_json_job(cwd, TRIGGER_CAPTURE_JOB, req)
            .map_err(TriggerCaptureError::from)
    }
}

const TRIGGER_CAPTURE_PY: &str = include_str!("../../scripts/trigger_capture.py");

const TRIGGER_CAPTURE_JOB: QRenderDocJsonJob =
    QRenderDocJsonJob::new("trigger_capture", "trigger_capture.py", TRIGGER_CAPTURE_PY);
