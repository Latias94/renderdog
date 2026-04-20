use std::path::Path;

use crate::{
    default_capture_basename, resolve_export_output_dir_from_cwd, resolve_path_string_from_cwd,
};

#[derive(Debug, Clone)]
pub(crate) struct PreparedExportTarget {
    pub capture_path: String,
    pub output_dir: String,
    pub basename: String,
}

pub(crate) fn normalize_capture_path(cwd: &Path, capture_path: &str) -> String {
    resolve_path_string_from_cwd(cwd, capture_path)
}

pub(crate) fn prepare_export_target(
    cwd: &Path,
    capture_path: &str,
    output_dir: Option<&str>,
    basename: Option<&str>,
) -> Result<PreparedExportTarget, std::io::Error> {
    let capture_path = normalize_capture_path(cwd, capture_path);
    let output_dir = resolve_export_output_dir_from_cwd(cwd, output_dir);
    std::fs::create_dir_all(&output_dir)?;

    Ok(PreparedExportTarget {
        basename: basename
            .map(str::to_owned)
            .unwrap_or_else(|| default_capture_basename(&capture_path)),
        capture_path,
        output_dir: output_dir.display().to_string(),
    })
}
