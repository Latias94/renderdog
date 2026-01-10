use std::path::{Path, PathBuf};

pub(crate) fn resolve_base_cwd(cwd: Option<String>) -> Result<PathBuf, String> {
    let current = std::env::current_dir().map_err(|e| format!("get cwd failed: {e}"))?;
    let Some(cwd) = cwd else {
        return Ok(current);
    };

    let p = PathBuf::from(cwd);
    if p.is_absolute() {
        Ok(p)
    } else {
        Ok(current.join(p))
    }
}

pub(crate) fn resolve_path_from_base(base: &Path, value: &str) -> PathBuf {
    let p = PathBuf::from(value);
    if p.is_absolute() { p } else { base.join(p) }
}
