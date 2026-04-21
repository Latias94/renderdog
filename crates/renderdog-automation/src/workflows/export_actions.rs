use std::path::Path;

use crate::qrenderdoc_jobs::EXPORT_ACTIONS_JOB;
use crate::{QRenderDocJobError, RenderDocInstallation};

use super::{ExportActionsRequest, ExportActionsResponse};

impl RenderDocInstallation {
    pub(super) fn export_actions(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, QRenderDocJobError> {
        self.run_qrenderdoc_job(cwd, EXPORT_ACTIONS_JOB, req)
    }
}
