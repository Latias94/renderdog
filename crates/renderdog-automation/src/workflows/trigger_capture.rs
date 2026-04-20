use std::path::Path;

use crate::qrenderdoc_jobs::TRIGGER_CAPTURE_JOB;
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
