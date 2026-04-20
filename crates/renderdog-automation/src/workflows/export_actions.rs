use std::path::Path;

use crate::scripting::QRenderDocJsonJob;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{ExportActionsRequest, ExportActionsResponse};

impl RenderDocInstallation {
    pub(super) fn export_actions_jsonl_prepared(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, QRenderDocJsonError> {
        self.run_qrenderdoc_json_job(cwd, EXPORT_ACTIONS_JSONL_JOB, req)
    }
}

const EXPORT_ACTIONS_JSONL_PY: &str = include_str!("../../scripts/export_actions_jsonl.py");

const EXPORT_ACTIONS_JSONL_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "export_actions_jsonl",
    "export_actions_jsonl.py",
    EXPORT_ACTIONS_JSONL_PY,
);
