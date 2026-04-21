use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use crate::RenderDocInstallation;
use crate::command::CommandError;
use crate::default_scripts_dir;
use crate::{CommandSpec, ToolInvocationError, run_command_expect_success};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct QRenderDocJobEnvelope<T> {
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

#[derive(Debug, Error)]
pub enum QRenderDocJobError {
    #[error("failed to create scripts dir: {0}")]
    CreateScriptsDir(std::io::Error),
    #[error("failed to write python script: {0}")]
    WriteScript(std::io::Error),
    #[error("failed to write request JSON: {0}")]
    WriteRequest(std::io::Error),
    #[error("qrenderdoc execution failed: {0}")]
    QRenderDocExecution(Box<QRenderDocExecutionError>),
    #[error("failed to read response JSON: {0}")]
    ReadResponse(std::io::Error),
    #[error("failed to parse JSON: {0}")]
    ParseJson(serde_json::Error),
    #[error("qrenderdoc script error: {0}")]
    ScriptError(String),
}

impl From<QRenderDocPythonError> for QRenderDocJobError {
    fn from(value: QRenderDocPythonError) -> Self {
        Self::QRenderDocExecution(Box::new(QRenderDocExecutionError::from(value)))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct QRenderDocScriptFile {
    pub file_name: &'static str,
    pub content: &'static str,
}

impl QRenderDocScriptFile {
    pub(crate) const fn new(file_name: &'static str, content: &'static str) -> Self {
        Self { file_name, content }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QRenderDocJob {
    pub run_dir_prefix: &'static str,
    pub script_file_name: &'static str,
    pub script_content: &'static str,
    pub support_files: &'static [QRenderDocScriptFile],
}

impl QRenderDocJob {
    pub(crate) const fn with_support_files(
        run_dir_prefix: &'static str,
        script_file_name: &'static str,
        script_content: &'static str,
        support_files: &'static [QRenderDocScriptFile],
    ) -> Self {
        Self {
            run_dir_prefix,
            script_file_name,
            script_content,
            support_files,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QRenderDocPythonRequest {
    pub script_path: PathBuf,
    pub args: Vec<OsString>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum QRenderDocExecutionError {
    #[error("script not found: {0}")]
    ScriptNotFound(PathBuf),
    #[error(transparent)]
    Tool(Box<ToolInvocationError>),
}

impl From<QRenderDocPythonError> for QRenderDocExecutionError {
    fn from(value: QRenderDocPythonError) -> Self {
        match value {
            QRenderDocPythonError::ScriptNotFound(path) => Self::ScriptNotFound(path),
            QRenderDocPythonError::Command(err) => Self::Tool(Box::new((*err).into())),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum QRenderDocPythonError {
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

pub(crate) trait PrepareQRenderDocJobRequest: Serialize + Sized {
    type Error: From<QRenderDocJobError>;

    fn prepare_in_cwd(&self, cwd: &Path) -> Result<Self, Self::Error>;
}

impl RenderDocInstallation {
    // Use this when the request is already normalized and ready to serialize as-is.
    pub(crate) fn run_qrenderdoc_job<TReq, TResp>(
        &self,
        cwd: &Path,
        job: QRenderDocJob,
        request: &TReq,
    ) -> Result<TResp, QRenderDocJobError>
    where
        TReq: Serialize,
        TResp: DeserializeOwned,
    {
        let scripts_dir = default_scripts_dir(cwd);
        std::fs::create_dir_all(&scripts_dir).map_err(QRenderDocJobError::CreateScriptsDir)?;

        let script_path = scripts_dir.join(job.script_file_name);
        write_script_file(&script_path, job.script_content)
            .map_err(QRenderDocJobError::WriteScript)?;
        for support_file in job.support_files {
            let support_path = scripts_dir.join(support_file.file_name);
            write_script_file(&support_path, support_file.content)
                .map_err(QRenderDocJobError::WriteScript)?;
        }

        let run_dir = create_qrenderdoc_run_dir(&scripts_dir, job.run_dir_prefix)
            .map_err(QRenderDocJobError::CreateScriptsDir)?;
        let job_file_stem = Path::new(job.script_file_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(job.script_file_name);
        let request_path = run_dir.join(format!("{job_file_stem}.request.json"));
        let response_path = run_dir.join(format!("{job_file_stem}.response.json"));
        remove_if_exists(&response_path).map_err(QRenderDocJobError::WriteRequest)?;

        std::fs::write(
            &request_path,
            serde_json::to_vec(request).map_err(QRenderDocJobError::ParseJson)?,
        )
        .map_err(QRenderDocJobError::WriteRequest)?;

        let _ = self.run_qrenderdoc_python(&QRenderDocPythonRequest {
            script_path,
            args: Vec::new(),
            working_dir: Some(run_dir),
        })?;

        let bytes = std::fs::read(&response_path).map_err(QRenderDocJobError::ReadResponse)?;
        let env: QRenderDocJobEnvelope<TResp> =
            serde_json::from_slice(&bytes).map_err(QRenderDocJobError::ParseJson)?;
        if env.ok {
            env.result
                .ok_or_else(|| QRenderDocJobError::ScriptError("missing result".into()))
        } else {
            Err(QRenderDocJobError::ScriptError(
                env.error.unwrap_or_else(|| "unknown error".into()),
            ))
        }
    }

    // Use this when the request depends on cwd-relative normalization or validation first.
    pub(crate) fn run_qrenderdoc_job_in_cwd<TReq, TResp>(
        &self,
        cwd: &Path,
        job: QRenderDocJob,
        request: &TReq,
    ) -> Result<TResp, TReq::Error>
    where
        TReq: PrepareQRenderDocJobRequest,
        TResp: DeserializeOwned,
    {
        let request = request.prepare_in_cwd(cwd)?;
        self.run_qrenderdoc_job(cwd, job, &request)
            .map_err(TReq::Error::from)
    }

    pub(crate) fn run_qrenderdoc_python(
        &self,
        req: &QRenderDocPythonRequest,
    ) -> Result<(), QRenderDocPythonError> {
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

        let _ = run_command_expect_success(&spec)?;

        Ok(())
    }
}

pub(crate) fn write_script_file(path: &Path, content: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content.as_bytes())
}

fn remove_if_exists(path: &Path) -> Result<(), std::io::Error> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}
