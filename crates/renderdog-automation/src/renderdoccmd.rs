use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::{CommandError, CommandSpec, run_command_expect_success, run_command_output_text};

#[derive(Debug, Clone)]
pub struct CaptureLaunchRequest {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub working_dir: Option<PathBuf>,
    pub capture_file_template: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct CaptureLaunchResult {
    pub target_ident: u32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Error)]
pub enum CaptureLaunchError {
    #[error(transparent)]
    Command(Box<CommandError>),
    #[error("renderdoccmd returned invalid target ident: {0}")]
    InvalidTargetIdent(i32),
}

impl From<CommandError> for CaptureLaunchError {
    fn from(value: CommandError) -> Self {
        Self::Command(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn launch_capture(
        &self,
        req: &CaptureLaunchRequest,
    ) -> Result<CaptureLaunchResult, CaptureLaunchError> {
        let mut spec = CommandSpec::new(&self.renderdoccmd_exe).arg("capture");

        if let Some(working_dir) = &req.working_dir {
            spec.args.push(OsString::from("-d"));
            spec.args.push(working_dir.as_os_str().to_owned());
        }

        if let Some(template) = &req.capture_file_template {
            spec.args.push(OsString::from("-c"));
            spec.args.push(template.as_os_str().to_owned());
        }

        spec.args.push(req.executable.as_os_str().to_owned());
        spec.args.extend(req.args.iter().cloned());

        let output = run_command_output_text(&spec)?;
        let stdout = output.stdout;
        let stderr = output.stderr;
        let code = output.status;
        let target_ident =
            u32::try_from(code).map_err(|_| CaptureLaunchError::InvalidTargetIdent(code))?;

        Ok(CaptureLaunchResult {
            target_ident,
            stdout,
            stderr,
        })
    }

    pub fn version(&self) -> Result<String, std::io::Error> {
        let spec = CommandSpec::new(&self.renderdoccmd_exe).arg("version");
        let output = run_command_output_text(&spec).map_err(|e| match e {
            CommandError::Spawn { source, .. } => source,
            other => std::io::Error::other(other.to_string()),
        })?;
        Ok(output.stdout)
    }

    pub fn save_thumbnail(
        &self,
        capture_path: &Path,
        output_path: &Path,
    ) -> Result<(), std::io::Error> {
        let spec = CommandSpec::new(&self.renderdoccmd_exe)
            .arg("thumb")
            .arg("-o")
            .arg(output_path.as_os_str().to_owned())
            .arg(capture_path.as_os_str().to_owned());

        match run_command_expect_success(&spec) {
            Ok(_) => Ok(()),
            Err(CommandError::Spawn { source, .. }) => Err(source),
            Err(other) => Err(std::io::Error::other(other.to_string())),
        }
    }
}
