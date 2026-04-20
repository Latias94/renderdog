use std::path::Path;

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::scripting::{QRenderDocJsonJobRequest, define_qrenderdoc_json_job_error};

use super::{FindEventsRequest, FindEventsResponse};

define_qrenderdoc_json_job_error! {
    #[derive(Debug, Error)]
    pub enum FindEventsError {
        create_dir_variant: CreateScriptsDir => "failed to create scripts dir: {0}",
        parse_json_message: "failed to parse JSON: {0}",
    }
}

impl RenderDocInstallation {
    pub fn find_events(
        &self,
        cwd: &Path,
        req: &FindEventsRequest,
    ) -> Result<FindEventsResponse, FindEventsError> {
        let req = FindEventsRequest {
            capture_path: crate::resolve_path_string_from_cwd(cwd, &req.capture_path),
            ..req.clone()
        };

        self.run_qrenderdoc_json_job(
            cwd,
            &QRenderDocJsonJobRequest {
                run_dir_prefix: "find_events",
                script_file_name: "find_events_json.py",
                script_content: FIND_EVENTS_JSON_PY,
                request: &req,
            },
        )
        .map_err(FindEventsError::from)
    }
}

const FIND_EVENTS_JSON_PY: &str = include_str!("../../scripts/find_events_json.py");
