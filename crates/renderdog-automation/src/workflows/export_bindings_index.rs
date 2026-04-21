use std::path::Path;

use crate::qrenderdoc_jobs::EXPORT_BINDINGS_INDEX_JOB;
use crate::{QRenderDocJobError, RenderDocInstallation};

use super::{ExportBindingsIndexRequest, ExportBindingsIndexResponse};

impl RenderDocInstallation {
    pub(super) fn export_bindings_index(
        &self,
        cwd: &Path,
        req: &ExportBindingsIndexRequest,
    ) -> Result<ExportBindingsIndexResponse, QRenderDocJobError> {
        self.run_qrenderdoc_job(cwd, EXPORT_BINDINGS_INDEX_JOB, req)
    }
}
