use crate::scripting::{QRenderDocJob, QRenderDocScriptFile};

const QRENDERDOC_RUNTIME_SUPPORT_FILE: QRenderDocScriptFile = QRenderDocScriptFile::new(
    "renderdog_qrenderdoc.py",
    include_str!("../scripts/renderdog_qrenderdoc.py"),
);

const ACTION_QUERY_SUPPORT_FILE: QRenderDocScriptFile = QRenderDocScriptFile::new(
    "renderdog_action_query.py",
    include_str!("../scripts/renderdog_action_query.py"),
);

const QRENDERDOC_RUNTIME_SUPPORT_FILES: &[QRenderDocScriptFile] =
    &[QRENDERDOC_RUNTIME_SUPPORT_FILE];

const ACTION_QUERY_SUPPORT_FILES: &[QRenderDocScriptFile] =
    &[QRENDERDOC_RUNTIME_SUPPORT_FILE, ACTION_QUERY_SUPPORT_FILE];

pub(crate) const EXPORT_ACTIONS_JSONL_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "export_actions_jsonl",
    "export_actions_jsonl.py",
    include_str!("../scripts/export_actions_jsonl.py"),
    ACTION_QUERY_SUPPORT_FILES,
);

pub(crate) const EXPORT_BINDINGS_INDEX_JSONL_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "export_bindings_index_jsonl",
    "export_bindings_index_jsonl.py",
    include_str!("../scripts/export_bindings_index_jsonl.py"),
    ACTION_QUERY_SUPPORT_FILES,
);

pub(crate) const FIND_EVENTS_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "find_events",
    "find_events_json.py",
    include_str!("../scripts/find_events_json.py"),
    ACTION_QUERY_SUPPORT_FILES,
);

pub(crate) const TRIGGER_CAPTURE_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "trigger_capture",
    "trigger_capture.py",
    include_str!("../scripts/trigger_capture.py"),
    QRENDERDOC_RUNTIME_SUPPORT_FILES,
);

pub(crate) const REPLAY_LIST_TEXTURES_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "replay_list_textures",
    "replay_list_textures_json.py",
    include_str!("../scripts/replay_list_textures_json.py"),
    QRENDERDOC_RUNTIME_SUPPORT_FILES,
);

pub(crate) const REPLAY_PICK_PIXEL_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "replay_pick_pixel",
    "replay_pick_pixel_json.py",
    include_str!("../scripts/replay_pick_pixel_json.py"),
    QRENDERDOC_RUNTIME_SUPPORT_FILES,
);

pub(crate) const REPLAY_SAVE_TEXTURE_PNG_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "replay_save_texture_png",
    "replay_save_texture_png_json.py",
    include_str!("../scripts/replay_save_texture_png_json.py"),
    QRENDERDOC_RUNTIME_SUPPORT_FILES,
);

pub(crate) const REPLAY_SAVE_OUTPUTS_PNG_JOB: QRenderDocJob = QRenderDocJob::with_support_files(
    "replay_save_outputs_png",
    "replay_save_outputs_png_json.py",
    include_str!("../scripts/replay_save_outputs_png_json.py"),
    QRENDERDOC_RUNTIME_SUPPORT_FILES,
);

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        EXPORT_ACTIONS_JSONL_JOB, EXPORT_BINDINGS_INDEX_JSONL_JOB, FIND_EVENTS_JOB,
        REPLAY_LIST_TEXTURES_JOB, REPLAY_PICK_PIXEL_JOB, REPLAY_SAVE_OUTPUTS_PNG_JOB,
        REPLAY_SAVE_TEXTURE_PNG_JOB, TRIGGER_CAPTURE_JOB,
    };

    #[test]
    fn qrenderdoc_job_registry_uses_unique_prefixes_and_script_names() {
        let jobs = [
            &EXPORT_ACTIONS_JSONL_JOB,
            &EXPORT_BINDINGS_INDEX_JSONL_JOB,
            &FIND_EVENTS_JOB,
            &TRIGGER_CAPTURE_JOB,
            &REPLAY_LIST_TEXTURES_JOB,
            &REPLAY_PICK_PIXEL_JOB,
            &REPLAY_SAVE_TEXTURE_PNG_JOB,
            &REPLAY_SAVE_OUTPUTS_PNG_JOB,
        ];

        let prefixes = jobs
            .iter()
            .map(|job| job.run_dir_prefix)
            .collect::<BTreeSet<_>>();
        let script_names = jobs
            .iter()
            .map(|job| job.script_file_name)
            .collect::<BTreeSet<_>>();

        assert_eq!(prefixes.len(), jobs.len());
        assert_eq!(script_names.len(), jobs.len());
    }

    #[test]
    fn action_query_jobs_bundle_shared_support_module() {
        let jobs = [
            &EXPORT_ACTIONS_JSONL_JOB,
            &EXPORT_BINDINGS_INDEX_JSONL_JOB,
            &FIND_EVENTS_JOB,
        ];

        for job in jobs {
            assert!(
                job.support_files
                    .iter()
                    .any(|file| file.file_name == "renderdog_action_query.py")
            );
        }
    }

    #[test]
    fn all_jobs_bundle_shared_qrenderdoc_runtime_module() {
        let jobs = [
            &EXPORT_ACTIONS_JSONL_JOB,
            &EXPORT_BINDINGS_INDEX_JSONL_JOB,
            &FIND_EVENTS_JOB,
            &TRIGGER_CAPTURE_JOB,
            &REPLAY_LIST_TEXTURES_JOB,
            &REPLAY_PICK_PIXEL_JOB,
            &REPLAY_SAVE_TEXTURE_PNG_JOB,
            &REPLAY_SAVE_OUTPUTS_PNG_JOB,
        ];

        for job in jobs {
            assert!(
                job.support_files
                    .iter()
                    .any(|file| file.file_name == "renderdog_qrenderdoc.py")
            );
        }
    }
}
