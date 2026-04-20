use std::path::Path;

use crate::qrenderdoc_jobs::EXPORT_BINDINGS_INDEX_JSONL_JOB;
use crate::{QRenderDocJsonError, RenderDocInstallation};

use super::{ExportBindingsIndexRequest, ExportBindingsIndexResponse};

impl RenderDocInstallation {
    pub(super) fn export_bindings_index_jsonl_prepared(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, QRenderDocJsonError> {
        self.run_prepared_qrenderdoc_json_job(cwd, EXPORT_BINDINGS_INDEX_JSONL_JOB, req)
    }
}
