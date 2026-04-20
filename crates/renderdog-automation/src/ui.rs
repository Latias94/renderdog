use std::{
    path::Path,
    process::{Child, Command},
};

use thiserror::Error;

use crate::RenderDocInstallation;
use crate::ToolInvocationError;
use crate::command::CommandError;

#[derive(Debug, Error)]
pub enum OpenCaptureUiError {
    #[error(transparent)]
    Tool(Box<ToolInvocationError>),
}

impl From<CommandError> for OpenCaptureUiError {
    fn from(value: CommandError) -> Self {
        Self::Tool(Box::new(value.into()))
    }
}

impl RenderDocInstallation {
    pub(crate) fn open_capture_in_ui(
        &self,
        capture_path: &Path,
    ) -> Result<Child, OpenCaptureUiError> {
        Command::new(&self.qrenderdoc_exe)
            .arg(capture_path)
            .spawn()
            .map_err(|e| {
                OpenCaptureUiError::Tool(Box::new(ToolInvocationError::from(CommandError::Spawn {
                    program: self.qrenderdoc_exe.display().to_string(),
                    args: vec![capture_path.display().to_string()],
                    cwd: None,
                    source: e,
                })))
            })
    }
}
