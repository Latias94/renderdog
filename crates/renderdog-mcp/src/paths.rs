use std::path::PathBuf;

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
