use std::path::Path;

use crate::scripting::QRenderDocJsonJob;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{TriggerCaptureRequest, TriggerCaptureResponse};

pub type TriggerCaptureError = QRenderDocJsonError;

impl RenderDocInstallation {
    pub fn trigger_capture_via_target_control(
        &self,
        cwd: &Path,
        req: &TriggerCaptureRequest,
    ) -> Result<TriggerCaptureResponse, TriggerCaptureError> {
        self.run_qrenderdoc_json_job(cwd, TRIGGER_CAPTURE_JOB, req)
    }
}

const TRIGGER_CAPTURE_PY: &str = include_str!("../../scripts/trigger_capture.py");

const TRIGGER_CAPTURE_JOB: QRenderDocJsonJob =
    QRenderDocJsonJob::new("trigger_capture", "trigger_capture.py", TRIGGER_CAPTURE_PY);
