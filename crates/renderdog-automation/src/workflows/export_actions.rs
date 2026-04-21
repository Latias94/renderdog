use std::path::Path;

use crate::qrenderdoc_jobs::EXPORT_ACTIONS_JSONL_JOB;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{ExportActionsRequest, ExportActionsResponse};

impl RenderDocInstallation {
    pub(super) fn export_actions_jsonl(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, QRenderDocJsonError> {
        self.run_qrenderdoc_json_job(cwd, EXPORT_ACTIONS_JSONL_JOB, req)
    }
}
