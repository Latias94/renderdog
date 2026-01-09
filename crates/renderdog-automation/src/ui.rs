use std::{
    path::Path,
    process::{Child, Command},
};

use thiserror::Error;

use crate::CommandError;
use crate::RenderDocInstallation;

#[derive(Debug, Error)]
pub enum OpenCaptureUiError {
    #[error(transparent)]
    Command(Box<CommandError>),
}

impl From<CommandError> for OpenCaptureUiError {
    fn from(value: CommandError) -> Self {
        Self::Command(Box::new(value))
    }
}

impl RenderDocInstallation {
    pub fn open_capture_in_ui(&self, capture_path: &Path) -> Result<Child, OpenCaptureUiError> {
        Command::new(&self.qrenderdoc_exe)
            .arg(capture_path)
            .spawn()
            .map_err(|e| {
                OpenCaptureUiError::Command(Box::new(CommandError::Spawn {
                    program: self.qrenderdoc_exe.display().to_string(),
                    args: vec![capture_path.display().to_string()],
                    cwd: None,
                    source: e,
                }))
            })
    }
}
