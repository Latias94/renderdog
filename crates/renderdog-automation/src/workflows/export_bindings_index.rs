use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{
    QRenderDocJsonJob, define_qrenderdoc_json_job_error_enum,
    impl_qrenderdoc_json_job_error_conversion,
};

use super::{ExportBindingsIndexRequest, ExportBindingsIndexResponse};

define_qrenderdoc_json_job_error_enum! {
    #[derive(Debug, Error)]
    pub(crate) enum ExportBindingsIndexError {
        create_dir_variant: CreateOutputDir => "failed to create output dir: {0}",
        parse_json_message: "failed to parse export JSON: {0}",
    }
}
impl_qrenderdoc_json_job_error_conversion!(
    ExportBindingsIndexError,
    create_dir_variant: CreateOutputDir,
);

impl RenderDocInstallation {
    pub(super) fn export_bindings_index_jsonl_prepared(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, ExportBindingsIndexError> {
        self.run_qrenderdoc_json_job(cwd, EXPORT_BINDINGS_INDEX_JSONL_JOB, req)
            .map_err(ExportBindingsIndexError::from)
    }
}

const EXPORT_BINDINGS_INDEX_JSONL_PY: &str =
    include_str!("../../scripts/export_bindings_index_jsonl.py");

const EXPORT_BINDINGS_INDEX_JSONL_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "export_bindings_index_jsonl",
    "export_bindings_index_jsonl.py",
    EXPORT_BINDINGS_INDEX_JSONL_PY,
);
