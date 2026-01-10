use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::{CommandError, CommandSpec, run_command_expect_success};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct QRenderDocJsonEnvelope<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub error: Option<String>,
}

pub(crate) fn create_qrenderdoc_run_dir(
    scripts_dir: &Path,
    prefix: &str,
) -> Result<PathBuf, std::io::Error> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);

    let runs_dir = scripts_dir.join("runs");
    std::fs::create_dir_all(&runs_dir)?;

    let run_dir = runs_dir.join(format!("{prefix}-{nanos}-{pid}-{seq}"));
    std::fs::create_dir_all(&run_dir)?;
    Ok(run_dir)
}

#[derive(Debug, Clone)]
pub struct QRenderDocPythonRequest {
    pub script_path: PathBuf,
    pub args: Vec<OsString>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct QRenderDocPythonResult {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

#[derive(Debug, Error)]
pub enum QRenderDocPythonError {
    #[error("script not found: {0}")]
    ScriptNotFound(PathBuf),
    #[error(transparent)]
    Command(Box<CommandError>),
}

impl From<CommandError> for QRenderDocPythonError {
    fn from(value: CommandError) -> Self {
        Self::Command(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn run_qrenderdoc_python(
        &self,
        req: &QRenderDocPythonRequest,
    ) -> Result<QRenderDocPythonResult, QRenderDocPythonError> {
        if !req.script_path.is_file() {
            return Err(QRenderDocPythonError::ScriptNotFound(
                req.script_path.clone(),
            ));
        }

        let mut spec = CommandSpec::new(&self.qrenderdoc_exe)
            .arg("--python")
            .arg(req.script_path.as_os_str().to_owned());
        spec.args.extend(req.args.iter().cloned());
        if let Some(wd) = &req.working_dir {
            spec.cwd = Some(wd.clone());
        }

        let output = run_command_expect_success(&spec)?;

        Ok(QRenderDocPythonResult {
            stdout: output.stdout,
            stderr: output.stderr,
            status: output.status,
        })
    }
}

pub fn write_script_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content.as_bytes())
}
