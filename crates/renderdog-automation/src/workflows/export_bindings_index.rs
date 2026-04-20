use std::path::Path;

use crate::scripting::QRenderDocJsonJob;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{ExportBindingsIndexRequest, ExportBindingsIndexResponse};

impl RenderDocInstallation {
    pub(super) fn export_bindings_index_jsonl_prepared(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, QRenderDocJsonError> {
        self.run_qrenderdoc_json_job(cwd, EXPORT_BINDINGS_INDEX_JSONL_JOB, req)
    }
}

const EXPORT_BINDINGS_INDEX_JSONL_PY: &str =
    include_str!("../../scripts/export_bindings_index_jsonl.py");

const EXPORT_BINDINGS_INDEX_JSONL_JOB: QRenderDocJsonJob = QRenderDocJsonJob::new(
    "export_bindings_index_jsonl",
    "export_bindings_index_jsonl.py",
    EXPORT_BINDINGS_INDEX_JSONL_PY,
);
