use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{
    QRenderDocJsonJob, define_qrenderdoc_json_job_error_enum,
    impl_qrenderdoc_json_job_error_conversion,
};

use super::{ExportActionsRequest, ExportActionsResponse};

define_qrenderdoc_json_job_error_enum! {
    #[derive(Debug, Error)]
    pub(crate) enum ExportActionsError {
        create_dir_variant: CreateOutputDir => "failed to create output dir: {0}",
        parse_json_message: "failed to parse export JSON: {0}",
    }
}
impl_qrenderdoc_json_job_error_conversion!(
    ExportActionsError,
    create_dir_variant: CreateOutputDir,
);

impl RenderDocInstallation {
    pub(super) fn export_actions_jsonl_prepared(
        &self,
        cwd: &Path,
        req: &ExportActionsRequest,
    ) -> Result<ExportActionsResponse, ExportActionsError> {
        self.run_qrenderdoc_json_job(cwd, EXPORT_ACTIONS_JSONL_JOB, req)
            .map_err(ExportActionsError::from)
    }
}

const EXPORT_ACTIONS_JSONL_PY: &str = include_str!("../../scripts/export_actions_jsonl.py");

const EXPORT_ACTIONS_JSONL_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "export_actions_jsonl",
    "export_actions_jsonl.py",
    EXPORT_ACTIONS_JSONL_PY,
);
